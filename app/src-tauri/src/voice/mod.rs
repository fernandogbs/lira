mod audio_thread;
mod clipboard;
mod stt_thread;

use std::sync::{mpsc, Arc};

use audio_thread::{AudioCommand, AudioEvent, AudioStartConfig};
use lira_core::history::History;
use lira_core::AppState;
use serde::Serialize;
use stt_thread::{SttCommand, SttEvent, TranscribeJob};
use tauri::{AppHandle, Emitter, Manager};

use crate::popup::{hide_popup, show_popup};
use crate::AppRuntime;

#[derive(Clone, Serialize)]
pub struct TranscriptPayload {
    pub text: String,
    pub latency_ms: i64,
    pub model: String,
}

#[derive(Clone, Serialize)]
pub struct ErrorPayload {
    pub message: String,
}

pub struct VoiceHandle {
    audio_tx: mpsc::Sender<AudioCommand>,
}

impl VoiceHandle {
    pub fn spawn(app: AppHandle, runtime: Arc<AppRuntime>) -> Self {
        let (audio_tx, audio_rx) = audio_thread::spawn_audio_thread();
        let (stt_tx, stt_rx) = stt_thread::spawn_stt_thread();
        let stt_tx_bridge = stt_tx.clone();

        let app_audio = app.clone();
        let runtime_audio = Arc::clone(&runtime);
        std::thread::spawn(move || {
            while let Ok(event) = audio_rx.recv() {
                match event {
                    AudioEvent::Started => {}
                    AudioEvent::Stopped(samples) => {
                        set_state(&app_audio, &runtime_audio, AppState::Transcribing);
                        let settings = runtime_audio.settings.lock().expect("settings").clone();
                        let _ = stt_tx_bridge
                            .send(SttCommand::Transcribe(TranscribeJob { samples, settings }));
                    }
                    AudioEvent::Error(message) => {
                        finish_error(&app_audio, &runtime_audio, message);
                    }
                }
            }
        });

        let app_stt = app.clone();
        let runtime_stt = Arc::clone(&runtime);
        std::thread::spawn(move || {
            while let Ok(event) = stt_rx.recv() {
                match event {
                    SttEvent::Skipped => {
                        set_state(&app_stt, &runtime_stt, AppState::Idle);
                    }
                    SttEvent::Done {
                        text,
                        latency_ms,
                        model,
                        language,
                        audio_ms,
                    } => {
                        clipboard::copy_to_clipboard(&text);
                        let settings = runtime_stt.settings.lock().expect("settings").clone();
                        if settings.history.enabled {
                            if let Ok(history) = History::open_default() {
                                if let Err(err) =
                                    history.insert(&text, &model, &language, audio_ms, latency_ms)
                                {
                                    log::warn!("history insert failed: {err}");
                                }
                            } else {
                                log::warn!("failed to open history database");
                            }
                        }
                        let payload = TranscriptPayload {
                            text,
                            latency_ms,
                            model,
                        };
                        let _ = app_stt.emit("lira://transcript", payload);
                        set_state(&app_stt, &runtime_stt, AppState::Idle);
                    }
                    SttEvent::Error(message) => {
                        finish_error(&app_stt, &runtime_stt, message);
                    }
                }
            }
        });

        Self { audio_tx }
    }

    pub fn start_recording(&self, app: &AppHandle, runtime: &Arc<AppRuntime>) {
        set_state(app, runtime, AppState::Listening);
        show_popup(app, runtime);
        let settings = runtime.settings.lock().expect("settings").clone();
        let cfg = AudioStartConfig {
            hotkey_mode: settings.hotkey.mode,
            vad_autostop_ms: settings.hotkey.vad_autostop_ms,
        };
        if let Err(err) = self.audio_tx.send(AudioCommand::Start(cfg)) {
            finish_error(app, runtime, err.to_string());
        }
    }

    pub fn stop_recording(&self, app: &AppHandle, runtime: &Arc<AppRuntime>) {
        if let Err(err) = self.audio_tx.send(AudioCommand::Stop) {
            finish_error(app, runtime, err.to_string());
        }
    }
}

fn finish_error(app: &AppHandle, runtime: &Arc<AppRuntime>, message: String) {
    log::warn!("voice error: {message}");
    let _ = app.emit(
        "lira://error",
        ErrorPayload {
            message: message.clone(),
        },
    );
    set_state(app, runtime, AppState::Idle);
}

fn set_state(app: &AppHandle, runtime: &Arc<AppRuntime>, state: AppState) {
    {
        let mut s = runtime.state.lock().expect("state");
        *s = state;
    }
    let app_main = app.clone();
    let runtime = Arc::clone(runtime);
    let _ = app.run_on_main_thread(move || {
        crate::emit_popup_status(&app_main, &runtime);
    });
}

pub fn handle_toggle(app: &AppHandle, runtime: &Arc<AppRuntime>, voice: &VoiceHandle) {
    let popup_open = app.get_webview_window("popup").is_some();
    let state = *runtime.state.lock().expect("state");
    match state {
        AppState::Transcribing => {}
        AppState::Listening => voice.stop_recording(app, runtime),
        AppState::Idle if popup_open => hide_popup(app),
        AppState::Idle => voice.start_recording(app, runtime),
    }
}

pub fn handle_start(app: &AppHandle, runtime: &Arc<AppRuntime>, voice: &VoiceHandle) {
    let state = *runtime.state.lock().expect("state");
    if state == AppState::Idle {
        voice.start_recording(app, runtime);
    }
}

pub fn handle_stop(app: &AppHandle, runtime: &Arc<AppRuntime>, voice: &VoiceHandle) {
    let state = *runtime.state.lock().expect("state");
    if state == AppState::Listening {
        voice.stop_recording(app, runtime);
    }
}
