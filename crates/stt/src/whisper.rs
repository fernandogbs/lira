use std::num::NonZeroUsize;
use std::sync::Arc;
use std::thread::available_parallelism;

use lira_core::settings::SttModel;
use thiserror::Error;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use crate::models::{ensure_model, ModelId, ModelSpec};

#[derive(Debug, Error)]
pub enum WhisperError {
    #[error("model: {0}")]
    Model(#[from] crate::models::ModelError),
    #[error("whisper: {0}")]
    Whisper(#[from] whisper_rs::WhisperError),
    #[error("failed to load whisper model at {0}")]
    LoadFailed(String),
}

pub struct WhisperEngine {
    ctx: Arc<WhisperContext>,
}

impl WhisperEngine {
    pub fn load(path: &str, use_gpu: bool) -> Result<Self, WhisperError> {
        let mut params = WhisperContextParameters::default();
        params.use_gpu(use_gpu);
        let ctx = WhisperContext::new_with_params(path, params)
            .map_err(|_| WhisperError::LoadFailed(path.to_string()))?;
        Ok(Self { ctx: Arc::new(ctx) })
    }

    pub fn load_model(spec: &ModelSpec, use_gpu: bool) -> Result<Self, WhisperError> {
        let path = ensure_model(spec)?;
        Self::load(path.to_string_lossy().as_ref(), use_gpu)
    }

    pub fn transcribe(
        &self,
        pcm_16k_mono_f32: &[f32],
        language: &str,
        initial_prompt: Option<&str>,
    ) -> Result<String, WhisperError> {
        let mut state = self.ctx.create_state()?;
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        let threads = available_parallelism()
            .unwrap_or(NonZeroUsize::new(4).unwrap())
            .get()
            .min(8) as i32;
        params.set_n_threads(threads);
        params.set_translate(false);
        params.set_language(Some(language));
        params.set_no_timestamps(true);
        params.set_suppress_nst(true);
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        if let Some(prompt) = initial_prompt {
            if !prompt.is_empty() {
                params.set_initial_prompt(prompt);
            }
        }

        state.full(params, pcm_16k_mono_f32)?;
        let n = state.full_n_segments();
        let mut parts = Vec::new();
        for i in 0..n {
            if let Some(segment) = state.get_segment(i) {
                let s = segment.to_str()?.trim();
                if !s.is_empty() {
                    parts.push(s);
                }
            }
        }
        Ok(parts.join(" "))
    }
}

pub fn load_with_fallback(
    primary: SttModel,
    fallback: SttModel,
    gpu: bool,
) -> Result<WhisperEngine, WhisperError> {
    let primary_spec = ModelId::from(primary).spec();
    match WhisperEngine::load_model(primary_spec, gpu) {
        Ok(engine) => Ok(engine),
        Err(primary_err) => {
            log::warn!(
                "failed to load STT model {:?}: {primary_err}; falling back to {:?}",
                primary,
                fallback
            );
            let fallback_spec = ModelId::from(fallback).spec();
            WhisperEngine::load_model(fallback_spec, gpu)
        }
    }
}
