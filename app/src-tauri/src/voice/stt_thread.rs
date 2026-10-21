use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::{Duration, Instant};

use lira_core::settings::{Settings, SttModel};
use lira_stt::{load_with_fallback, WhisperEngine};

use lira_audio::SAMPLE_RATE;

pub const MIN_AUDIO_SAMPLES: usize = (SAMPLE_RATE as f32 * 0.3) as usize;

pub struct TranscribeJob {
    pub samples: Vec<f32>,
    pub settings: Settings,
}

pub enum SttCommand {
    Transcribe(TranscribeJob),
}

pub enum SttEvent {
    Skipped,
    Done {
        text: String,
        latency_ms: i64,
        model: String,
        language: String,
        audio_ms: i64,
    },
    Error(String),
}

struct EngineSlot {
    engine: WhisperEngine,
    primary: SttModel,
    fallback: SttModel,
    gpu: bool,
}

pub fn spawn_stt_thread() -> (mpsc::Sender<SttCommand>, mpsc::Receiver<SttEvent>) {
    let (cmd_tx, cmd_rx) = mpsc::channel();
    let (event_tx, event_rx) = mpsc::channel();
    std::thread::spawn(move || run(cmd_rx, event_tx));
    (cmd_tx, event_rx)
}

fn run(cmd_rx: mpsc::Receiver<SttCommand>, event_tx: mpsc::Sender<SttEvent>) {
    let mut slot: Option<EngineSlot> = None;
    let mut last_used: Option<Instant> = None;
    let mut last_keep_warm_min = 5u32;
    const IDLE_POLL: Duration = Duration::from_secs(30);

    loop {
        let cmd = match cmd_rx.recv_timeout(IDLE_POLL) {
            Ok(cmd) => cmd,
            Err(RecvTimeoutError::Timeout) => {
                maybe_unload(&mut slot, &mut last_used, last_keep_warm_min);
                continue;
            }
            Err(RecvTimeoutError::Disconnected) => break,
        };
        match cmd {
            SttCommand::Transcribe(job) => {
                last_keep_warm_min = job.settings.stt.keep_warm_min;
                maybe_unload(&mut slot, &mut last_used, last_keep_warm_min);
                if job.samples.len() < MIN_AUDIO_SAMPLES {
                    let _ = event_tx.send(SttEvent::Skipped);
                    continue;
                }
                let audio_ms = (job.samples.len() as i64) * 1000 / i64::from(SAMPLE_RATE);
                let language = job.settings.effective_language();
                let prompt = job.settings.initial_prompt();

                let started = Instant::now();
                if let Err(err) = ensure_engine(
                    &mut slot,
                    job.settings.stt.model,
                    job.settings.stt.fallback_model,
                    job.settings.stt.gpu,
                ) {
                    let _ = event_tx.send(SttEvent::Error(err));
                    continue;
                }

                let engine = &slot.as_ref().expect("engine loaded").engine;
                let model_label = model_label(job.settings.stt.model);
                match engine.transcribe(
                    &job.samples,
                    language,
                    prompt.as_deref(),
                    job.settings.stt.audio_ctx_auto,
                ) {
                    Ok(text) => {
                        last_used = Some(Instant::now());
                        let latency_ms = started.elapsed().as_millis() as i64;
                        let _ = event_tx.send(SttEvent::Done {
                            text,
                            latency_ms,
                            model: model_label,
                            language: language.to_string(),
                            audio_ms,
                        });
                    }
                    Err(err) => {
                        let _ = event_tx.send(SttEvent::Error(err.to_string()));
                    }
                }
            }
        }
    }
}

fn model_label(model: SttModel) -> String {
    match model {
        SttModel::LargeV3TurboQ5_0 => "large-v3-turbo-q5_0".to_string(),
        SttModel::SmallQ5_1 => "small-q5_1".to_string(),
    }
}

fn maybe_unload(
    slot: &mut Option<EngineSlot>,
    last_used: &mut Option<Instant>,
    keep_warm_min: u32,
) {
    if slot.is_none() {
        return;
    }
    let Some(last) = *last_used else {
        return;
    };
    let warm = Duration::from_secs(keep_warm_min as u64 * 60);
    if last.elapsed() >= warm {
        *slot = None;
        *last_used = None;
        log::info!("stt: unloaded whisper engine after keep_warm_min");
    }
}

fn ensure_engine(
    slot: &mut Option<EngineSlot>,
    primary: SttModel,
    fallback: SttModel,
    gpu: bool,
) -> Result<(), String> {
    let needs_reload = match slot.as_ref() {
        None => true,
        Some(s) => s.primary != primary || s.fallback != fallback || s.gpu != gpu,
    };
    if needs_reload {
        *slot = None;
        let engine = load_with_fallback(primary, fallback, gpu).map_err(|e| e.to_string())?;
        *slot = Some(EngineSlot {
            engine,
            primary,
            fallback,
            gpu,
        });
    }
    Ok(())
}
