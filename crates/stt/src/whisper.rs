use std::sync::Mutex;

use lira_core::settings::SttModel;
use thiserror::Error;
use whisper_rs::{
    FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, WhisperState,
};

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

const WHISPER_SAMPLE_RATE: f64 = 16_000.0;
/// Max whisper `audio_ctx` (30 s at 20 ms/frame).
const AUDIO_CTX_MAX: i32 = 1500;

/// Whisper `audio_ctx` sized from 16 kHz mono sample count (20 ms frames, cap 30 s).
pub fn audio_ctx_for(samples_len: usize) -> i32 {
    let secs = samples_len as f64 / WHISPER_SAMPLE_RATE;
    let frames = secs.mul_add(50.0, 0.0).ceil() as i32 + 64;
    round_up_to_multiple_of_64(frames).min(AUDIO_CTX_MAX)
}

fn round_up_to_multiple_of_64(n: i32) -> i32 {
    ((n + 63) / 64) * 64
}

/// Thread count for whisper.cpp (physical cores by default, max 8).
pub fn stt_n_threads() -> usize {
    if let Ok(s) = std::env::var("LIRA_STT_THREADS") {
        if let Ok(n) = s.parse::<usize>() {
            if n >= 1 {
                return n;
            }
        }
    }
    num_cpus::get_physical().clamp(1, 8)
}

fn stt_env_flag(name: &str) -> bool {
    std::env::var(name).is_ok_and(|v| v == "1")
}

pub struct WhisperEngine {
    ctx: WhisperContext,
    state: Mutex<WhisperState>,
}

impl WhisperEngine {
    pub fn load(path: &str, use_gpu: bool) -> Result<Self, WhisperError> {
        eprintln!("lira-stt: {}", whisper_rs::print_system_info());
        let mut params = WhisperContextParameters::default();
        params.use_gpu(use_gpu);
        params.flash_attn(!std::env::var("LIRA_STT_FLASH_ATTN").is_ok_and(|v| v == "0"));
        let ctx = WhisperContext::new_with_params(path, params)
            .map_err(|_| WhisperError::LoadFailed(path.to_string()))?;
        let state = ctx.create_state()?;
        Ok(Self {
            ctx,
            state: Mutex::new(state),
        })
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
        audio_ctx_auto: bool,
    ) -> Result<String, WhisperError> {
        let mut state = self.state.lock().expect("whisper state mutex poisoned");
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_n_threads(stt_n_threads() as i32);
        params.set_translate(false);
        params.set_language(Some(language));
        params.set_no_timestamps(true);
        params.set_suppress_nst(true);
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_no_context(true);
        params.set_single_segment(false);
        if stt_env_flag("LIRA_STT_NO_FALLBACK") {
            params.set_temperature_inc(0.0);
        }
        if let Some(prompt) = initial_prompt {
            if !prompt.is_empty() {
                params.set_initial_prompt(prompt);
            }
        }
        if audio_ctx_auto {
            params.set_audio_ctx(audio_ctx_for(pcm_16k_mono_f32.len()));
        }

        let log_timings = stt_env_flag("LIRA_STT_TIMINGS");
        if log_timings {
            self.ctx.reset_timings();
        }
        state.full(params, pcm_16k_mono_f32)?;
        if log_timings {
            eprintln!("lira-stt: whisper.cpp timings:");
            self.ctx.print_timings();
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audio_ctx_for_empty_is_64() {
        assert_eq!(audio_ctx_for(0), 64);
    }

    #[test]
    fn audio_ctx_for_one_second() {
        assert_eq!(audio_ctx_for(16_000), 128);
    }

    #[test]
    fn audio_ctx_for_ten_seconds() {
        assert_eq!(audio_ctx_for(160_000), 576);
    }

    #[test]
    fn audio_ctx_for_long_audio_caps_at_1500_then_rounds() {
        assert_eq!(audio_ctx_for(16_000 * 60), 1500);
    }
}
