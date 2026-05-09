use std::io::Write;
use std::process::{Command, Stdio};

use crate::error::{AppError, AppResult};

pub fn copy_text(text: &str) -> AppResult<()> {
    let mut child = Command::new("wl-copy")
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|source| AppError::CommandSpawn {
            command: "wl-copy".to_string(),
            source,
        })?;

    let stdin = child
        .stdin
        .as_mut()
        .ok_or_else(|| AppError::ProcessFailed {
            command: "wl-copy".to_string(),
            stderr: "failed to open stdin".to_string(),
        })?;
    stdin.write_all(text.as_bytes())?;

    let output = child.wait_with_output()?;
    if output.status.success() {
        Ok(())
    } else {
        Err(AppError::ProcessFailed {
            command: "wl-copy".to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        })
    }
}
