//! OCR via Tesseract (`leptess`).

use thiserror::Error;

#[derive(Debug, Error)]
pub enum OcrError {
    #[error("{0}")]
    Msg(String),
}

/// Run Tesseract on PNG bytes. `langs` like `"por+eng"`.
pub fn ocr_png(png: &[u8], langs: &str) -> Result<String, OcrError> {
    let mut tess = leptess::LepTess::new(None, langs)
        .map_err(|e| OcrError::Msg(format!("tesseract init ({langs}): {e}")))?;
    tess.set_image_from_mem(png)
        .map_err(|e| OcrError::Msg(format!("set_image: {e}")))?;
    let text = tess
        .get_utf8_text()
        .map_err(|e| OcrError::Msg(format!("get_utf8_text: {e}")))?;
    Ok(text.trim().to_string())
}
