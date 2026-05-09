use std::fs;

use crate::clipboard;
use crate::error::AppResult;
use crate::ocr::TesseractOcr;
use crate::screenshot::PortalScreenshot;

pub type ExtractionResult = AppResult<ExtractedText>;

#[derive(Debug, Clone)]
pub struct ExtractedText {
    pub text: String,
    pub copied_to_clipboard: bool,
}

#[derive(Default)]
pub struct TextExtractor;

impl TextExtractor {
    pub fn extract(&self, languages: &str) -> ExtractionResult {
        let screenshot = PortalScreenshot::capture_interactive()?;
        let text = TesseractOcr::extract_text(&screenshot, languages);
        let _ = fs::remove_file(&screenshot);

        let text = text?;
        let copied_to_clipboard = clipboard::copy_text(&text).is_ok();

        Ok(ExtractedText {
            text,
            copied_to_clipboard,
        })
    }
}
