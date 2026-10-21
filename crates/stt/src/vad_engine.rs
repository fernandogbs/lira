use whisper_rs::{WhisperVadContext, WhisperVadContextParams, WhisperVadParams};

use crate::models::{ensure_model, VAD_SILERO};

pub struct VadEngine {
    ctx: WhisperVadContext,
}

impl VadEngine {
    pub fn try_open() -> Option<Self> {
        let path = match ensure_model(&VAD_SILERO) {
            Ok(p) => p,
            Err(e) => {
                log::warn!("VAD model unavailable: {e}");
                return None;
            }
        };
        WhisperVadContext::new(
            path.to_string_lossy().as_ref(),
            WhisperVadContextParams::default(),
        )
        .map_err(|e| log::warn!("failed to open VAD context: {e}"))
        .ok()
        .map(|ctx| Self { ctx })
    }

    pub fn has_speech(&mut self, samples: &[f32]) -> bool {
        let params = WhisperVadParams::default();
        match self.ctx.segments_from_samples(params, samples) {
            Ok(segments) => segments.num_segments() > 0,
            Err(_) => false,
        }
    }
}
