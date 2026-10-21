//! Post-STT agent dispatch (intent router → Claude Code).

use std::sync::Arc;

use futures_util::StreamExt;
use lira_adapter_api::{AdapterEvent, AgentAdapter};
use lira_adapter_claude_code::ClaudeCodeAdapter;
use lira_core::policy::{classify_action, decide_voice_approval, PolicyDecision};
use lira_core::router::{route, Intent};
use lira_core::settings::Settings;
use lira_core::AppState;
use lira_store::{SessionRow, Store};
use notify_rust::Notification;
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::{emit_popup_status, AppRuntime};

#[derive(Clone, Serialize)]
struct AgentReplyPayload {
    session: String,
    text: String,
}

pub struct AgentHandle {
    adapter: Arc<dyn AgentAdapter>,
}

impl AgentHandle {
    pub fn spawn(app: AppHandle, runtime: Arc<AppRuntime>) -> Self {
        let adapter: Arc<dyn AgentAdapter> = Arc::new(ClaudeCodeAdapter::new());
        let events_adapter = Arc::clone(&adapter);
        let app_ev = app.clone();
        let runtime_ev = Arc::clone(&runtime);
        tauri::async_runtime::spawn(async move {
            let mut stream = events_adapter.events();
            while let Some(event) = stream.next().await {
                match event {
                    AdapterEvent::Message { session, body } => {
                        let locale = runtime_ev.locale;
                        let _ = app_ev.emit(
                            "lira://agent-reply",
                            AgentReplyPayload {
                                session: session.clone(),
                                text: body.clone(),
                            },
                        );
                        let _ = Notification::new()
                            .summary(locale.catalog().agent_done)
                            .body(&truncate(&body, 180))
                            .show();
                        set_idle(&app_ev, &runtime_ev);
                    }
                    AdapterEvent::StatusChanged(_) => {}
                    _ => {}
                }
            }
        });
        Self { adapter }
    }

    pub fn handle_transcript(&self, app: &AppHandle, runtime: &Arc<AppRuntime>, text: &str) {
        let settings = runtime.settings.lock().expect("settings").clone();
        if !settings.agent.enabled {
            return;
        }
        if text.trim().is_empty() {
            return;
        }

        let intent = route(text);
        let adapter = Arc::clone(&self.adapter);
        let app = app.clone();
        let runtime = Arc::clone(runtime);
        tauri::async_runtime::spawn(async move {
            if let Err(err) = dispatch(adapter, &app, &runtime, &settings, intent).await {
                log::warn!("agent dispatch: {err}");
                let _ = app.emit("lira://error", crate::voice::ErrorPayload { message: err });
                set_idle(&app, &runtime);
            }
        });
    }
}

async fn dispatch(
    adapter: Arc<dyn AgentAdapter>,
    app: &AppHandle,
    runtime: &Arc<AppRuntime>,
    settings: &Settings,
    intent: Intent,
) -> Result<(), String> {
    let locale = runtime.locale;
    let status = adapter.status().await?;
    if !status.ready {
        return Err(locale.catalog().agent_not_ready.to_string());
    }

    match intent {
        Intent::Cancel => {
            set_idle(app, runtime);
            Ok(())
        }
        Intent::Approve { target } => {
            let class = classify_action(&target);
            match decide_voice_approval(settings.policy.voice_approve, class) {
                PolicyDecision::DenyVoice => {
                    Err(locale.catalog().agent_denied_destructive.to_string())
                }
                PolicyDecision::RequireClick => Err(locale.catalog().agent_needs_click.to_string()),
                PolicyDecision::Allow => {
                    let session = resolve_session(&adapter, settings, None).await?;
                    set_waiting(app, runtime);
                    let msg = format!("User approved by voice: {target}");
                    adapter.send(&session, &msg).await?;
                    touch_store(&session, "Approved");
                    Ok(())
                }
            }
        }
        Intent::Status { session } => {
            let session = resolve_session(&adapter, settings, session.as_deref()).await?;
            set_waiting(app, runtime);
            adapter
                .send(
                    &session,
                    "Brief status: what are you doing and any blockers?",
                )
                .await?;
            touch_store(&session, "Status");
            Ok(())
        }
        Intent::Summarize { session } => {
            let session = resolve_session(&adapter, settings, session.as_deref()).await?;
            set_waiting(app, runtime);
            let summary = adapter.summarize(&session).await?;
            let _ = app.emit(
                "lira://agent-reply",
                AgentReplyPayload {
                    session: session.clone(),
                    text: summary.clone(),
                },
            );
            let _ = Notification::new()
                .summary(locale.catalog().agent_done)
                .body(&truncate(&summary, 180))
                .show();
            set_idle(app, runtime);
            Ok(())
        }
        Intent::Instruct { session, message } => {
            if message.is_empty() {
                return Ok(());
            }
            let class = classify_action(&message);
            if matches!(
                decide_voice_approval(settings.policy.voice_approve, class),
                PolicyDecision::DenyVoice
            ) {
                return Err(locale.catalog().agent_denied_destructive.to_string());
            }
            let session = resolve_session(&adapter, settings, session.as_deref()).await?;
            set_waiting(app, runtime);
            let lang = settings.effective_language();
            let prompt = if lang == "pt" {
                format!("Responda em português brasileiro.\n\n{message}")
            } else {
                format!("Reply in English.\n\n{message}")
            };
            adapter.send(&session, &prompt).await?;
            touch_store(&session, &truncate(&message, 48));
            Ok(())
        }
    }
}

async fn resolve_session(
    adapter: &Arc<dyn AgentAdapter>,
    settings: &Settings,
    explicit: Option<&str>,
) -> Result<String, String> {
    if let Some(name) = explicit {
        let sessions = adapter.list_sessions().await?;
        if let Some(s) = sessions
            .iter()
            .find(|s| s.id == name || s.title.eq_ignore_ascii_case(name))
        {
            return Ok(s.id.clone());
        }
        return Ok(name.to_string());
    }

    let default = settings.agent.default_session.as_str();
    if default != "focused" && !default.is_empty() {
        return Ok(default.to_string());
    }

    // focused: prefer most recently used from store, else first discovered, else "new"
    if let Ok(store) = Store::open_default() {
        if let Ok(list) = store.list_sessions() {
            if let Some(s) = list.first() {
                return Ok(s.id.clone());
            }
        }
    }
    let sessions = adapter.list_sessions().await?;
    Ok(sessions
        .first()
        .map(|s| s.id.clone())
        .unwrap_or_else(|| "new".into()))
}

fn touch_store(id: &str, title: &str) {
    if let Ok(store) = Store::open_default() {
        let _ = store.upsert_session(&SessionRow {
            id: id.to_string(),
            title: title.to_string(),
            adapter: "claude-code".into(),
            cwd: None,
            last_used_at: chrono::Utc::now().to_rfc3339(),
        });
    }
}

fn set_waiting(app: &AppHandle, runtime: &Arc<AppRuntime>) {
    {
        let mut s = runtime.state.lock().expect("state");
        *s = AppState::WaitingAgent;
    }
    emit_popup_status(app, runtime);
}

fn set_idle(app: &AppHandle, runtime: &Arc<AppRuntime>) {
    {
        let mut s = runtime.state.lock().expect("state");
        *s = AppState::Idle;
    }
    emit_popup_status(app, runtime);
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}
