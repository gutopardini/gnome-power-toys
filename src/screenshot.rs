use gtk::gio::{self, prelude::*};
use gtk::glib::{self, Variant};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::error::{AppError, AppResult};

const PORTAL_BUS: &str = "org.freedesktop.portal.Desktop";
const PORTAL_OBJECT: &str = "/org/freedesktop/portal/desktop";
const SCREENSHOT_INTERFACE: &str = "org.freedesktop.portal.Screenshot";
const REQUEST_INTERFACE: &str = "org.freedesktop.portal.Request";

pub struct PortalScreenshot;

#[derive(Debug)]
enum PortalResponse {
    Success(PathBuf),
    Cancelled,
    Failed(String),
}

impl PortalScreenshot {
    pub fn capture_interactive() -> AppResult<PathBuf> {
        let context = glib::MainContext::new();
        context
            .with_thread_default(capture_interactive_on_context)
            .map_err(|error| AppError::ProcessFailed {
                command: "xdg-desktop-portal Screenshot".to_string(),
                stderr: error.to_string(),
            })?
    }
}

fn capture_interactive_on_context() -> AppResult<PathBuf> {
    let connection =
        gio::bus_get_sync(gio::BusType::Session, gio::Cancellable::NONE).map_err(|error| {
            AppError::ProcessFailed {
                command: "D-Bus session bus".to_string(),
                stderr: error.to_string(),
            }
        })?;

    let token = request_token();
    let request_path = expected_request_path(&connection, &token)?;
    let (tx, rx) = mpsc::channel();

    let subscription = connection.subscribe_to_signal(
        Some(PORTAL_BUS),
        Some(REQUEST_INTERFACE),
        Some("Response"),
        Some(&request_path),
        None,
        gio::DBusSignalFlags::NONE,
        move |signal| {
            let _ = tx.send(parse_response(signal.parameters));
        },
    );

    let options = glib::VariantDict::new(None);
    options.insert("interactive", true);
    options.insert("handle_token", token.as_str());
    let parameters = ("", options).to_variant();

    let call_result = connection.call_sync(
        Some(PORTAL_BUS),
        PORTAL_OBJECT,
        SCREENSHOT_INTERFACE,
        "Screenshot",
        Some(&parameters),
        None,
        gio::DBusCallFlags::NONE,
        -1,
        gio::Cancellable::NONE,
    );

    if let Err(error) = call_result {
        drop(subscription);
        return Err(AppError::ProcessFailed {
            command: "xdg-desktop-portal Screenshot".to_string(),
            stderr: error.to_string(),
        });
    }

    let response = wait_for_response(&rx)?;
    drop(subscription);

    match response {
        PortalResponse::Success(path) => Ok(path),
        PortalResponse::Cancelled => Err(AppError::ProcessFailed {
            command: "xdg-desktop-portal Screenshot".to_string(),
            stderr: "screenshot was cancelled".to_string(),
        }),
        PortalResponse::Failed(error) => Err(AppError::ProcessFailed {
            command: "xdg-desktop-portal Screenshot".to_string(),
            stderr: error,
        }),
    }
}

fn wait_for_response(rx: &mpsc::Receiver<PortalResponse>) -> AppResult<PortalResponse> {
    let started = Instant::now();
    let context = glib::MainContext::thread_default().ok_or_else(|| AppError::ProcessFailed {
        command: "xdg-desktop-portal Screenshot".to_string(),
        stderr: "missing GLib thread-default context".to_string(),
    })?;

    loop {
        while context.pending() {
            context.iteration(false);
        }

        match rx.try_recv() {
            Ok(response) => return Ok(response),
            Err(mpsc::TryRecvError::Disconnected) => {
                return Err(AppError::ProcessFailed {
                    command: "xdg-desktop-portal Screenshot".to_string(),
                    stderr: "portal response listener disconnected".to_string(),
                });
            }
            Err(mpsc::TryRecvError::Empty) => {}
        }

        if started.elapsed() > Duration::from_secs(120) {
            return Err(AppError::ProcessFailed {
                command: "xdg-desktop-portal Screenshot".to_string(),
                stderr: "timed out waiting for portal response".to_string(),
            });
        }

        std::thread::sleep(Duration::from_millis(20));
    }
}

fn parse_response(parameters: &Variant) -> PortalResponse {
    let Some((response_code, results)) = parameters.get::<(u32, BTreeMap<String, Variant>)>()
    else {
        return PortalResponse::Failed(format!(
            "could not parse portal response: {}",
            parameters.print(true)
        ));
    };

    if response_code != 0 {
        return PortalResponse::Cancelled;
    }

    let Some(uri_variant) = results.get("uri") else {
        return PortalResponse::Failed("portal response did not include an image URI".to_string());
    };

    let Some(uri) = uri_variant.get::<String>() else {
        return PortalResponse::Failed("portal image URI had an unexpected type".to_string());
    };

    match uri_to_path(&uri) {
        Ok(path) => PortalResponse::Success(path),
        Err(error) => PortalResponse::Failed(error.to_string()),
    }
}

fn expected_request_path(connection: &gio::DBusConnection, token: &str) -> AppResult<String> {
    let unique_name = connection
        .unique_name()
        .ok_or_else(|| AppError::ProcessFailed {
            command: "D-Bus session bus".to_string(),
            stderr: "connection does not have a unique bus name".to_string(),
        })?;

    Ok(format!(
        "/org/freedesktop/portal/desktop/request/{}/{}",
        sanitize_unique_name(unique_name.as_str()),
        token
    ))
}

fn request_token() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();

    format!("gnome_power_toys_{millis}")
}

fn sanitize_unique_name(unique_name: &str) -> String {
    unique_name
        .trim_start_matches(':')
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '_'
            }
        })
        .collect()
}

fn uri_to_path(uri: &str) -> AppResult<PathBuf> {
    let Some(path) = uri.strip_prefix("file://") else {
        return Err(AppError::ProcessFailed {
            command: "xdg-desktop-portal Screenshot".to_string(),
            stderr: format!("portal returned unsupported URI: {uri}"),
        });
    };

    Ok(PathBuf::from(percent_decode(path)?))
}

fn percent_decode(value: &str) -> AppResult<String> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[index + 1..index + 3]).map_err(|error| {
                AppError::ProcessFailed {
                    command: "xdg-desktop-portal Screenshot".to_string(),
                    stderr: error.to_string(),
                }
            })?;
            if let Ok(byte) = u8::from_str_radix(hex, 16) {
                decoded.push(byte);
                index += 3;
                continue;
            }
        }

        decoded.push(bytes[index]);
        index += 1;
    }

    String::from_utf8(decoded).map_err(|error| AppError::ProcessFailed {
        command: "xdg-desktop-portal Screenshot".to_string(),
        stderr: error.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizes_unique_name_for_portal_path() {
        assert_eq!(sanitize_unique_name(":1.148"), "1_148");
    }

    #[test]
    fn decodes_file_uri() {
        assert_eq!(
            uri_to_path("file:///tmp/Gnome%20Power%20Toys.png").unwrap(),
            PathBuf::from("/tmp/Gnome Power Toys.png")
        );
    }
}
