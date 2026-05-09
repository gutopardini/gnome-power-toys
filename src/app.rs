use gtk::gio;
use gtk::prelude::*;
use gtk::{
    Align, Application, ApplicationWindow, Box, Button, CssProvider, DropDown, EventControllerKey,
    HeaderBar, Image, Label, Orientation, ScrolledWindow, TextBuffer, TextView, gdk,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;
use std::time::Duration;

use crate::text_extractor::{ExtractionResult, TextExtractor};

const APP_ID: &str = "dev.gutopardini.GnomePowerToys";
const APP_ICON: &[u8] = include_bytes!("../assets/app-icons/gnome-power-toys-icon.svg");

pub fn run() -> gtk::glib::ExitCode {
    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.connect_startup(|_| install_css());
    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    let app_hold = Rc::new(RefCell::new(Some(app.hold())));

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Gnome Power Toys")
        .default_width(980)
        .default_height(620)
        .build();
    window.add_css_class("gpt-window");

    let header_title = Label::builder()
        .label("Gnome Power Toys")
        .css_classes(["title-3"])
        .build();
    header_title.add_css_class("gpt-header-title");

    let header = HeaderBar::builder().title_widget(&header_title).build();
    header.add_css_class("gpt-header");
    header.pack_start(&app_icon_image());
    window.set_titlebar(Some(&header));

    let root = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(0)
        .margin_top(18)
        .margin_bottom(18)
        .margin_start(18)
        .margin_end(18)
        .build();
    root.add_css_class("gpt-shell");

    let sidebar = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .width_request(230)
        .build();
    sidebar.add_css_class("gpt-sidebar");

    let brand = Label::builder()
        .label("Gnome Power Toys")
        .halign(Align::Start)
        .css_classes(["title-3"])
        .build();
    brand.add_css_class("gpt-brand");

    let text_extractor_tool = tool_button("Text Extractor", true);
    let color_picker_tool = tool_button("Color Picker  Soon", false);

    sidebar.append(&brand);
    sidebar.append(&text_extractor_tool);
    sidebar.append(&color_picker_tool);

    let content = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(16)
        .hexpand(true)
        .build();
    content.add_css_class("gpt-content");

    let title = Label::builder()
        .label("Text Extractor")
        .halign(Align::Start)
        .css_classes(["title-2"])
        .build();

    let controls = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .halign(Align::Fill)
        .build();
    controls.add_css_class("gpt-toolbar");

    let language_dropdown =
        DropDown::from_strings(&["Portuguese + English", "Portuguese", "English"]);
    language_dropdown.set_selected(0);
    language_dropdown.set_tooltip_text(Some("OCR language"));
    language_dropdown.add_css_class("gpt-entry");

    let control_spacer = Box::builder().hexpand(true).build();

    let extract_button = Button::builder()
        .label("Extract text")
        .css_classes(["suggested-action"])
        .build();
    extract_button.add_css_class("gpt-extract-button");

    controls.append(&language_dropdown);
    controls.append(&control_spacer);
    controls.append(&extract_button);

    let status = Label::builder()
        .label("Ready")
        .halign(Align::Start)
        .wrap(true)
        .build();
    status.add_css_class("gpt-status");

    let result_header = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .halign(Align::Fill)
        .build();
    result_header.add_css_class("gpt-result-header");

    let result_title = Label::builder()
        .label("Extracted text")
        .halign(Align::Start)
        .hexpand(true)
        .css_classes(["heading"])
        .build();

    let copy_button = Button::from_icon_name("edit-copy-symbolic");
    copy_button.set_tooltip_text(Some("Copy extracted text"));
    copy_button.set_sensitive(false);
    copy_button.add_css_class("gpt-icon-button");
    copy_button.add_css_class("gpt-copy-button");

    result_header.append(&result_title);
    result_header.append(&copy_button);

    let text_buffer = TextBuffer::new(None);
    let text_view = TextView::builder()
        .buffer(&text_buffer)
        .editable(false)
        .monospace(true)
        .wrap_mode(gtk::WrapMode::WordChar)
        .vexpand(true)
        .build();
    text_view.add_css_class("gpt-result-text");

    let scroller = ScrolledWindow::builder()
        .child(&text_view)
        .vexpand(true)
        .hexpand(true)
        .build();
    scroller.add_css_class("gpt-result-frame");

    content.append(&title);
    content.append(&controls);
    content.append(&status);
    content.append(&result_header);
    content.append(&scroller);

    root.append(&sidebar);
    root.append(&content);
    window.set_child(Some(&root));

    wire_text_extractor(
        &window,
        &extract_button,
        &copy_button,
        &language_dropdown,
        &status,
        &text_buffer,
    );

    let app_hold_for_close = Rc::clone(&app_hold);
    window.connect_close_request(move |_| {
        app_hold_for_close.borrow_mut().take();
        gtk::glib::Propagation::Proceed
    });

    window.present();
}

fn tool_button(label: &str, active: bool) -> Button {
    let button = Button::builder().label(label).halign(Align::Fill).build();
    button.add_css_class("gpt-tool-button");

    if active {
        button.add_css_class("gpt-tool-active");
    } else {
        button.set_sensitive(false);
    }

    button
}

fn app_icon_image() -> Image {
    let bytes = gtk::glib::Bytes::from_static(APP_ICON);
    let image = match gdk::Texture::from_bytes(&bytes) {
        Ok(texture) => Image::from_paintable(Some(&texture)),
        Err(_) => Image::from_icon_name(APP_ID),
    };
    image.set_pixel_size(38);
    image.add_css_class("gpt-header-icon");
    image
}

fn wire_text_extractor(
    window: &ApplicationWindow,
    extract_button: &Button,
    copy_button: &Button,
    language_dropdown: &DropDown,
    status: &Label,
    text_buffer: &TextBuffer,
) {
    let pending_rx: Rc<RefCell<Option<mpsc::Receiver<ExtractionResult>>>> =
        Rc::new(RefCell::new(None));
    let last_text = Rc::new(RefCell::new(String::new()));

    {
        let pending_rx = Rc::clone(&pending_rx);
        let last_text = Rc::clone(&last_text);
        let status = status.clone();
        let text_buffer = text_buffer.clone();
        let extract_button = extract_button.clone();
        let copy_button = copy_button.clone();
        let window = window.clone();

        gtk::glib::timeout_add_local(Duration::from_millis(120), move || {
            let mut pending = pending_rx.borrow_mut();
            let Some(rx) = pending.as_mut() else {
                return gtk::glib::ControlFlow::Continue;
            };

            match rx.try_recv() {
                Ok(result) => {
                    window.present();
                    apply_result(
                        &status,
                        &text_buffer,
                        &extract_button,
                        &copy_button,
                        &last_text,
                        result,
                    );
                    pending.take();
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    window.present();
                    status.set_text("Extraction failed: worker disconnected");
                    extract_button.set_sensitive(true);
                    copy_button.set_sensitive(!last_text.borrow().is_empty());
                    pending.take();
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }

            gtk::glib::ControlFlow::Continue
        });
    }

    {
        let pending_rx = Rc::clone(&pending_rx);
        let status = status.clone();
        let text_buffer = text_buffer.clone();
        let button_for_callback = extract_button.clone();
        let button_for_state = extract_button.clone();
        let copy_button = copy_button.clone();
        let language_dropdown = language_dropdown.clone();
        let window = window.clone();

        button_for_callback.connect_clicked(move |_| {
            if pending_rx.borrow().is_some() {
                return;
            }

            let languages = ocr_language_code(language_dropdown.selected()).to_string();

            text_buffer.set_text("");
            status.set_text("Preparing screen selection...");
            button_for_state.set_sensitive(false);
            copy_button.set_sensitive(false);
            window.set_visible(false);

            let (tx, rx) = mpsc::channel();
            pending_rx.borrow_mut().replace(rx);

            gtk::glib::timeout_add_local_once(Duration::from_millis(180), move || {
                std::thread::spawn(move || {
                    let result = TextExtractor::default().extract(&languages);
                    let _ = tx.send(result);
                });
            });
        });
    }

    {
        let status = status.clone();
        let last_text = Rc::clone(&last_text);
        let window = window.clone();
        let button_for_callback = copy_button.clone();
        let button_for_feedback = copy_button.clone();
        button_for_callback.connect_clicked(move |_| {
            copy_current_text(&status, &last_text, &window, &button_for_feedback);
        });
    }

    {
        let status = status.clone();
        let last_text = Rc::clone(&last_text);
        let window = window.clone();
        let window_for_shortcut = window.clone();
        let copy_button = copy_button.clone();
        let key_controller = EventControllerKey::new();
        key_controller.connect_key_pressed(move |_, key, _, state| {
            if state.contains(gdk::ModifierType::CONTROL_MASK) && key == gdk::Key::c {
                copy_current_text(&status, &last_text, &window_for_shortcut, &copy_button);
                return gtk::glib::Propagation::Stop;
            }

            gtk::glib::Propagation::Proceed
        });
        window.add_controller(key_controller);
    }
}

fn apply_result(
    status: &Label,
    text_buffer: &TextBuffer,
    extract_button: &Button,
    copy_button: &Button,
    last_text: &Rc<RefCell<String>>,
    result: ExtractionResult,
) {
    extract_button.set_sensitive(true);

    match result {
        Ok(extracted) => {
            text_buffer.set_text(&extracted.text);
            last_text.replace(extracted.text);
            copy_button.set_sensitive(true);
            if extracted.copied_to_clipboard {
                status.set_text("Text extracted and copied to clipboard");
            } else {
                status.set_text("Text extracted. Install wl-clipboard to copy automatically.");
            }
        }
        Err(error) => {
            last_text.borrow_mut().clear();
            copy_button.set_sensitive(false);
            status.set_text(&format!("Extraction failed: {error}"));
        }
    }
}

fn copy_current_text(
    status: &Label,
    last_text: &Rc<RefCell<String>>,
    window: &ApplicationWindow,
    copy_button: &Button,
) {
    window.present();

    let text = last_text.borrow().clone();
    if text.trim().is_empty() {
        status.set_text("No extracted text to copy");
        return;
    }

    match crate::clipboard::copy_text(&text) {
        Ok(()) => {
            status.set_text("Text copied to clipboard");
            show_copied_feedback(copy_button);
        }
        Err(error) => status.set_text(&format!("Copy failed: {error}")),
    }
}

fn show_copied_feedback(copy_button: &Button) {
    copy_button.add_css_class("gpt-copy-success");

    let copy_button = copy_button.clone();
    gtk::glib::timeout_add_local_once(Duration::from_millis(900), move || {
        copy_button.remove_css_class("gpt-copy-success");
    });
}

fn install_css() {
    let Some(display) = gdk::Display::default() else {
        return;
    };

    let provider = CssProvider::new();
    provider.load_from_string(
        r#"
        window.gpt-window {
            background: rgba(18, 20, 24, 0.92);
        }

        .gpt-header {
            background: rgba(18, 20, 24, 0.90);
            border-bottom: 1px solid rgba(255, 255, 255, 0.14);
        }

        .gpt-header-icon {
            margin-left: 6px;
            margin-right: 12px;
            border-radius: 8px;
        }

        .gpt-header-title {
            color: #ffffff;
            font-weight: 700;
        }

        .gpt-shell {
            background:
                linear-gradient(135deg, rgba(48, 55, 66, 0.50), rgba(15, 17, 22, 0.92));
            color: #f3f4f7;
            border-radius: 10px;
        }

        .gpt-sidebar {
            background: rgba(255, 255, 255, 0.055);
            border: 1px solid rgba(255, 255, 255, 0.10);
            border-radius: 8px 0 0 8px;
            padding: 14px;
        }

        .gpt-content {
            background: rgba(8, 10, 14, 0.34);
            border-top: 1px solid rgba(255, 255, 255, 0.08);
            border-right: 1px solid rgba(255, 255, 255, 0.08);
            border-bottom: 1px solid rgba(255, 255, 255, 0.08);
            border-radius: 0 8px 8px 0;
            padding: 18px;
        }

        .gpt-brand {
            margin-bottom: 10px;
        }

        .gpt-tool-button {
            min-height: 38px;
            padding: 8px 10px;
            border-radius: 6px;
            background: transparent;
            color: rgba(243, 244, 247, 0.76);
            border: 1px solid transparent;
        }

        .gpt-tool-button label {
            margin-left: 2px;
        }

        .gpt-tool-active {
            background: rgba(255, 255, 255, 0.12);
            color: #ffffff;
            border-color: rgba(255, 255, 255, 0.16);
        }

        .gpt-toolbar,
        .gpt-result-frame {
            background: rgba(255, 255, 255, 0.075);
            border: 1px solid rgba(255, 255, 255, 0.12);
            border-radius: 8px;
            box-shadow: 0 14px 34px rgba(0, 0, 0, 0.26);
        }

        .gpt-toolbar {
            padding: 8px;
        }

        .gpt-entry {
            background: rgba(0, 0, 0, 0.20);
            border-radius: 6px;
            min-width: 190px;
        }

        .gpt-status {
            color: rgba(243, 244, 247, 0.78);
        }

        .gpt-result-header {
            margin-top: 4px;
        }

        .gpt-extract-button {
            min-width: 132px;
            font-weight: 700;
        }

        .gpt-icon-button {
            min-width: 34px;
            min-height: 34px;
            border-radius: 6px;
        }

        .gpt-copy-button {
            background: rgba(47, 211, 162, 0.20);
            color: #dfffee;
            border: 1px solid rgba(82, 235, 186, 0.38);
            box-shadow: 0 8px 22px rgba(47, 211, 162, 0.16);
        }

        .gpt-copy-button:hover {
            background: rgba(47, 211, 162, 0.32);
            border-color: rgba(112, 255, 205, 0.55);
        }

        .gpt-copy-success {
            background: rgba(82, 235, 186, 0.58);
            color: #07130f;
            border-color: rgba(186, 255, 229, 0.90);
            box-shadow: 0 0 0 3px rgba(82, 235, 186, 0.22), 0 12px 34px rgba(47, 211, 162, 0.34);
        }

        .gpt-result-frame {
            padding: 1px;
        }

        textview.gpt-result-text,
        textview.gpt-result-text text {
            background: rgba(10, 12, 15, 0.60);
            color: #f7f7f8;
            caret-color: #ffffff;
        }
        "#,
    );

    gtk::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn ocr_language_code(selected: u32) -> &'static str {
    match selected {
        1 => "por",
        2 => "eng",
        _ => "por+eng",
    }
}

#[cfg(test)]
mod tests {
    use super::ocr_language_code;

    #[test]
    fn maps_language_dropdown_to_tesseract_codes() {
        assert_eq!(ocr_language_code(0), "por+eng");
        assert_eq!(ocr_language_code(1), "por");
        assert_eq!(ocr_language_code(2), "eng");
        assert_eq!(ocr_language_code(99), "por+eng");
    }
}
