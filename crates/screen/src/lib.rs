//! Opt-in screen context: capture → OCR/redact (or raw image) in memory only.

pub mod capture;
pub mod exclude;
pub mod ocr;
pub mod redact;

use capture::{capture_for_context, CaptureError, CaptureResult};
use exclude::is_excluded;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScreenError {
    #[error(transparent)]
    Capture(#[from] CaptureError),
    #[error("{0}")]
    Msg(String),
    #[error("excluded app: {0}")]
    Excluded(String),
}

#[derive(Debug, Clone)]
pub enum ScreenPayload {
    Text(String),
    ImagePng(Vec<u8>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenMode {
    Text,
    Image,
}

/// Capture once for the current utterance. Never writes the image to durable storage.
pub fn capture_context(
    mode: ScreenMode,
    scope_window: bool,
    exclude_apps: &[String],
) -> Result<ScreenPayload, ScreenError> {
    let CaptureResult { png_bytes, focused } = capture_for_context(scope_window)?;
    if let Some(ref fw) = focused {
        if is_excluded(&fw.app_id, exclude_apps) {
            return Err(ScreenError::Excluded(fw.app_id.clone()));
        }
    }

    match mode {
        ScreenMode::Image => Ok(ScreenPayload::ImagePng(png_bytes)),
        ScreenMode::Text => {
            let raw = ocr::ocr_png(&png_bytes, "por+eng")
                .or_else(|_| ocr::ocr_png(&png_bytes, "eng"))
                .map_err(|e| ScreenError::Msg(e.to_string()))?;
            Ok(ScreenPayload::Text(redact::redact(&raw)))
        }
    }
}

pub use exclude::niri_block_out_snippet;
