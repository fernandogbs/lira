//! Local speech-to-text via whisper.cpp (`whisper-rs`).

mod decision;
mod models;
mod vad_engine;
mod whisper;

pub use decision::choose_default;
pub use models::{
    ensure_model, model_path, models_dir, verify_file, ModelError, ModelId, ModelSpec,
    LARGE_V3_TURBO_Q5_0, SMALL_Q5_1, VAD_SILERO,
};
pub use vad_engine::VadEngine;
pub use whisper::{audio_ctx_for, load_with_fallback, stt_n_threads, WhisperEngine, WhisperError};
