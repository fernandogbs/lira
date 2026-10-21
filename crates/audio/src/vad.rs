use std::time::{Duration, Instant};

use lira_stt::VadEngine;

pub const SAMPLE_RATE: u32 = 16_000;
const MAX_VAD_WINDOW_SECS: f32 = 3.0;
const VAD_MIN_INTERVAL: Duration = Duration::from_millis(250);

/// RMS threshold for the energy fallback (normalized f32 samples).
pub const ENERGY_SPEECH_THRESHOLD: f32 = 0.02;

pub struct SilenceDetector {
    vad: Option<VadEngine>,
    last_vad_speech_at: Option<Instant>,
    cached_speech_vad: bool,
    last_vad_tail_at: Option<Instant>,
    cached_tail_vad: bool,
    saw_speech: bool,
}

impl Default for SilenceDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl SilenceDetector {
    pub fn new() -> Self {
        Self {
            vad: try_load_vad(),
            last_vad_speech_at: None,
            cached_speech_vad: false,
            last_vad_tail_at: None,
            cached_tail_vad: false,
            saw_speech: false,
        }
    }

    pub fn with_energy_fallback_only() -> Self {
        Self {
            vad: None,
            last_vad_speech_at: None,
            cached_speech_vad: false,
            last_vad_tail_at: None,
            cached_tail_vad: false,
            saw_speech: false,
        }
    }

    /// `true` when speech was observed earlier and the last `silence_ms` contain no speech.
    pub fn speech_ended(&mut self, samples: &[f32], silence_ms: u32) -> bool {
        if samples.is_empty() || silence_ms == 0 {
            return false;
        }

        self.update_speech_seen(samples);
        if !self.saw_speech {
            return false;
        }

        let silence_samples = (silence_ms as usize) * (SAMPLE_RATE as usize) / 1000;
        if samples.len() < silence_samples {
            return false;
        }
        let tail = &samples[samples.len() - silence_samples..];
        !segment_has_speech(self, tail)
    }

    fn update_speech_seen(&mut self, samples: &[f32]) {
        if self.saw_speech {
            return;
        }
        if let Some(vad) = self.vad.as_mut() {
            let window = tail_window(samples, MAX_VAD_WINDOW_SECS);
            if vad_has_speech(
                vad,
                window,
                &mut self.last_vad_speech_at,
                &mut self.cached_speech_vad,
            ) {
                self.saw_speech = true;
            }
        } else if energy_has_speech(samples) {
            self.saw_speech = true;
        }
    }
}

fn try_load_vad() -> Option<VadEngine> {
    VadEngine::try_open()
}

fn tail_window(samples: &[f32], max_secs: f32) -> &[f32] {
    let max_samples = (max_secs * SAMPLE_RATE as f32) as usize;
    if samples.len() <= max_samples {
        samples
    } else {
        &samples[samples.len() - max_samples..]
    }
}

fn vad_has_speech(
    vad: &mut VadEngine,
    window: &[f32],
    last_at: &mut Option<Instant>,
    cached: &mut bool,
) -> bool {
    let now = Instant::now();
    if let Some(last) = *last_at {
        if now.duration_since(last) < VAD_MIN_INTERVAL {
            return *cached;
        }
    }
    *last_at = Some(now);
    *cached = vad.has_speech(window);
    *cached
}

fn segment_has_speech(detector: &mut SilenceDetector, segment: &[f32]) -> bool {
    if let Some(vad) = detector.vad.as_mut() {
        vad_has_speech(
            vad,
            segment,
            &mut detector.last_vad_tail_at,
            &mut detector.cached_tail_vad,
        )
    } else {
        energy_has_speech(segment)
    }
}

pub fn energy_has_speech(samples: &[f32]) -> bool {
    if samples.is_empty() {
        return false;
    }
    let sum_sq = samples.iter().map(|s| s * s).sum::<f32>();
    let rms = (sum_sq / samples.len() as f32).sqrt();
    rms >= ENERGY_SPEECH_THRESHOLD
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    fn sine(freq_hz: f32, secs: f32, amp: f32) -> Vec<f32> {
        let n = (secs * SAMPLE_RATE as f32) as usize;
        (0..n)
            .map(|i| {
                let t = i as f32 / SAMPLE_RATE as f32;
                amp * (2.0 * PI * freq_hz * t).sin()
            })
            .collect()
    }

    fn silence(secs: f32) -> Vec<f32> {
        vec![0.0; (secs * SAMPLE_RATE as f32) as usize]
    }

    #[test]
    fn energy_detects_sine() {
        let s = sine(440.0, 0.2, 0.5);
        assert!(energy_has_speech(&s));
        assert!(!energy_has_speech(&silence(0.2)));
    }

    #[test]
    fn speech_ended_after_trailing_silence() {
        let mut det = SilenceDetector::with_energy_fallback_only();
        let mut buf = sine(440.0, 0.3, 0.5);
        buf.extend(silence(0.5));
        assert!(det.speech_ended(&buf, 400));
    }

    #[test]
    fn speech_not_ended_without_prior_speech() {
        let mut det = SilenceDetector::with_energy_fallback_only();
        let buf = silence(1.0);
        assert!(!det.speech_ended(&buf, 400));
    }

    #[test]
    fn speech_not_ended_when_still_talking() {
        let mut det = SilenceDetector::with_energy_fallback_only();
        let buf = sine(440.0, 1.0, 0.5);
        assert!(!det.speech_ended(&buf, 200));
    }
}
