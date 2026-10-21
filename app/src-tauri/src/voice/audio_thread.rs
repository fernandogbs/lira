use std::sync::mpsc;
use std::time::{Duration, Instant};

use lira_audio::{Recorder, SilenceDetector};
use lira_core::settings::HotkeyMode;

const POLL: Duration = Duration::from_millis(250);
const MAX_RECORDING: Duration = Duration::from_secs(120);
const RECENT_MS: u32 = 3000;

pub struct AudioStartConfig {
    pub hotkey_mode: HotkeyMode,
    pub vad_autostop_ms: u32,
}

pub enum AudioCommand {
    Start(AudioStartConfig),
    Stop,
}

pub enum AudioEvent {
    Started,
    Stopped(Vec<f32>),
    Error(String),
}

pub fn spawn_audio_thread() -> (mpsc::Sender<AudioCommand>, mpsc::Receiver<AudioEvent>) {
    let (cmd_tx, cmd_rx) = mpsc::channel();
    let (event_tx, event_rx) = mpsc::channel();
    std::thread::spawn(move || run(cmd_rx, event_tx));
    (cmd_tx, event_rx)
}

struct Session {
    detector: SilenceDetector,
    vad_autostop_ms: u32,
    vad_enabled: bool,
    started_at: Instant,
}

fn run(cmd_rx: mpsc::Receiver<AudioCommand>, event_tx: mpsc::Sender<AudioEvent>) {
    let mut recorder = Recorder::new();
    let mut session: Option<Session> = None;

    while let Ok(cmd) = cmd_rx.recv() {
        match cmd {
            AudioCommand::Start(config) => {
                if session.is_some() {
                    continue;
                }
                let vad_enabled =
                    config.hotkey_mode == HotkeyMode::Toggle && config.vad_autostop_ms > 0;
                let detector = SilenceDetector::new();
                if let Err(err) = recorder.start() {
                    let _ = event_tx.send(AudioEvent::Error(err.to_string()));
                    continue;
                }
                session = Some(Session {
                    detector,
                    vad_autostop_ms: config.vad_autostop_ms,
                    vad_enabled,
                    started_at: Instant::now(),
                });
                let _ = event_tx.send(AudioEvent::Started);
                poll_loop(&mut recorder, &cmd_rx, &event_tx, &mut session);
            }
            AudioCommand::Stop => {
                if session.is_some() {
                    finish(&mut recorder, &event_tx, &mut session);
                }
            }
        }
    }
}

fn poll_loop(
    recorder: &mut Recorder,
    cmd_rx: &mpsc::Receiver<AudioCommand>,
    event_tx: &mpsc::Sender<AudioEvent>,
    session: &mut Option<Session>,
) {
    while session.is_some() {
        match cmd_rx.recv_timeout(POLL) {
            Ok(AudioCommand::Stop) => {
                finish(recorder, event_tx, session);
                return;
            }
            Ok(AudioCommand::Start(_)) => {}
            Err(mpsc::RecvTimeoutError::Timeout) => {
                let Some(sess) = session.as_mut() else {
                    return;
                };
                if sess.started_at.elapsed() >= MAX_RECORDING {
                    finish(recorder, event_tx, session);
                    return;
                }
                if sess.vad_enabled {
                    let tail = recorder.recent_16k(RECENT_MS);
                    if sess.detector.speech_ended(&tail, sess.vad_autostop_ms) {
                        finish(recorder, event_tx, session);
                        return;
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                session.take();
                return;
            }
        }
    }
}

fn finish(
    recorder: &mut Recorder,
    event_tx: &mpsc::Sender<AudioEvent>,
    session: &mut Option<Session>,
) {
    session.take();
    let samples = recorder.stop();
    let _ = event_tx.send(AudioEvent::Stopped(samples));
}
