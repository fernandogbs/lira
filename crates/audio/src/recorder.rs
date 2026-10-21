use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat};
use thiserror::Error;

use crate::resample::resample_to_16k_mono;

#[derive(Debug, Error)]
pub enum RecordError {
    #[error("no input device")]
    NoInputDevice,
    #[error("no supported input config")]
    NoConfig,
    #[error("cpal: {0}")]
    Cpal(String),
}

/// Live capture buffer. `cpal::Stream` is `!Send`, so keep this `Recorder` on one thread.
pub struct Recorder {
    buffer: Arc<Mutex<Vec<f32>>>,
    sample_rate: Arc<Mutex<u32>>,
    stream: Option<cpal::Stream>,
}

impl Default for Recorder {
    fn default() -> Self {
        Self::new()
    }
}

impl Recorder {
    pub fn new() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(Vec::new())),
            sample_rate: Arc::new(Mutex::new(16_000)),
            stream: None,
        }
    }

    pub fn start(&mut self) -> Result<(), RecordError> {
        if self.stream.is_some() {
            return Ok(());
        }

        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or(RecordError::NoInputDevice)?;
        let config = device
            .default_input_config()
            .map_err(|e| RecordError::Cpal(e.to_string()))?;
        let sample_rate = config.sample_rate().0;
        *self.sample_rate.lock().unwrap() = sample_rate;
        self.buffer.lock().unwrap().clear();

        let buffer = Arc::clone(&self.buffer);
        let err_fn = |e| log::error!("audio stream error: {e}");

        let stream = match config.sample_format() {
            SampleFormat::F32 => build_stream::<f32>(&device, &config.into(), buffer, err_fn)?,
            SampleFormat::I16 => build_stream::<i16>(&device, &config.into(), buffer, err_fn)?,
            SampleFormat::U16 => build_stream::<u16>(&device, &config.into(), buffer, err_fn)?,
            other => {
                return Err(RecordError::Cpal(format!(
                    "unsupported sample format {other}"
                )));
            }
        };

        stream
            .play()
            .map_err(|e| RecordError::Cpal(e.to_string()))?;
        self.stream = Some(stream);
        Ok(())
    }

    /// Last `ms` of captured audio resampled to 16 kHz mono (for live VAD while recording).
    pub fn recent_16k(&self, ms: u32) -> Vec<f32> {
        if ms == 0 {
            return Vec::new();
        }
        let rate = *self.sample_rate.lock().unwrap();
        if rate == 0 {
            return Vec::new();
        }
        let raw_count = ((ms as u64) * rate as u64 / 1000) as usize;
        let buf = self.buffer.lock().unwrap();
        let slice = if buf.len() <= raw_count {
            buf.as_slice()
        } else {
            &buf[buf.len() - raw_count..]
        };
        resample_to_16k_mono(slice, rate)
    }

    /// Approximate number of 16 kHz mono samples captured so far.
    pub fn elapsed_samples_16k(&self) -> usize {
        let raw_len = self.buffer.lock().unwrap().len();
        let rate = *self.sample_rate.lock().unwrap();
        if rate == 0 || raw_len == 0 {
            return 0;
        }
        ((raw_len as u64) * 16_000 / rate as u64) as usize
    }

    pub fn stop(&mut self) -> Vec<f32> {
        if let Some(stream) = self.stream.take() {
            drop(stream);
        }
        let raw = self.buffer.lock().unwrap().clone();
        self.buffer.lock().unwrap().clear();
        let rate = *self.sample_rate.lock().unwrap();
        resample_to_16k_mono(&raw, rate)
    }
}

fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    buffer: Arc<Mutex<Vec<f32>>>,
    err_fn: impl Fn(cpal::StreamError) + Send + 'static,
) -> Result<cpal::Stream, RecordError>
where
    T: Sample + cpal::SizedSample,
    f32: cpal::FromSample<T>,
{
    let channels = config.channels as usize;
    device
        .build_input_stream(
            config,
            move |data: &[T], _| {
                let mut mono: Vec<f32> = Vec::with_capacity(data.len() / channels);
                for frame in data.chunks(channels) {
                    let sum = frame.iter().map(|s| f32::from_sample(*s)).sum::<f32>();
                    mono.push(sum / channels as f32);
                }
                buffer.lock().unwrap().extend(mono);
            },
            err_fn,
            None,
        )
        .map_err(|e| RecordError::Cpal(e.to_string()))
}
