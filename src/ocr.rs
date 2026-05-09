use std::path::Path;
use std::process::Command;

use crate::error::{AppError, AppResult};

pub struct TesseractOcr;

impl TesseractOcr {
    pub fn extract_text(image_path: &Path, languages: &str) -> AppResult<String> {
        let output = Command::new("tesseract")
            .arg(image_path)
            .arg("stdout")
            .arg("-l")
            .arg(languages)
            .arg("--psm")
            .arg("6")
            .output()
            .map_err(|source| {
                if source.kind() == std::io::ErrorKind::NotFound {
                    AppError::MissingDependency {
                        command: "tesseract".to_string(),
                        install_hint:
                            "sudo dnf install tesseract tesseract-langpack-por tesseract-langpack-eng"
                                .to_string(),
                    }
                } else {
                    AppError::CommandSpawn {
                        command: "tesseract".to_string(),
                        source,
                    }
                }
            })?;

        if !output.status.success() {
            return Err(AppError::ProcessFailed {
                command: "tesseract".to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            });
        }

        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if text.is_empty() {
            Err(AppError::OcrProducedNoText)
        } else {
            Ok(text)
        }
    }
}
