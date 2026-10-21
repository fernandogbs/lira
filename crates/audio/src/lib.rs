//! Audio capture (cpal) and voice-activity helpers.

mod recorder;
mod resample;
mod vad;

pub use recorder::{RecordError, Recorder};
pub use resample::resample_to_16k_mono;
pub use vad::{energy_has_speech, SilenceDetector, ENERGY_SPEECH_THRESHOLD, SAMPLE_RATE};
