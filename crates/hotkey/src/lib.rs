//! Hotkey backends for hold mode (portal GlobalShortcuts).
//!
//! Compositor binds (`lira start`/`stop`) and `lira-hold` (evdev) live outside this crate.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use ashpd::desktop::global_shortcuts::{GlobalShortcuts, NewShortcut};
use futures_util::StreamExt;
use log::{info, warn};
use tokio::sync::mpsc;

/// Press / release events from a hold backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyEvent {
    Pressed,
    Released,
}

const SHORTCUT_ID: &str = "lira-hold";

/// Spawn a background task that listens for portal activate/deactivate.
/// Returns a receiver of events. If the portal cannot be opened, logs a warning
/// and returns an empty channel that never fires (caller keeps CLI binds).
pub fn spawn_portal_hold(preferred_trigger: Option<&str>) -> mpsc::UnboundedReceiver<HotkeyEvent> {
    let (tx, rx) = mpsc::unbounded_channel();
    let trigger = preferred_trigger.unwrap_or("<Super>space").to_string();
    let running = Arc::new(AtomicBool::new(true));

    tokio::spawn(async move {
        if let Err(err) = run_portal_loop(&trigger, tx, Arc::clone(&running)).await {
            warn!("global shortcuts portal unavailable: {err}");
        }
        running.store(false, Ordering::SeqCst);
    });

    rx
}

async fn run_portal_loop(
    trigger: &str,
    tx: mpsc::UnboundedSender<HotkeyEvent>,
    running: Arc<AtomicBool>,
) -> Result<(), String> {
    let proxy = GlobalShortcuts::new()
        .await
        .map_err(|e| format!("GlobalShortcuts::new: {e}"))?;

    let session = proxy
        .create_session()
        .await
        .map_err(|e| format!("create_session: {e}"))?;

    let shortcut = NewShortcut::new(SHORTCUT_ID, "Lira hold-to-talk").preferred_trigger(trigger);

    proxy
        .bind_shortcuts(&session, &[shortcut], None)
        .await
        .map_err(|e| format!("bind_shortcuts: {e}"))?;

    info!("portal GlobalShortcuts bound (id={SHORTCUT_ID}, preferred={trigger})");

    let mut activated = proxy
        .receive_activated()
        .await
        .map_err(|e| format!("receive_activated: {e}"))?;
    let mut deactivated = proxy
        .receive_deactivated()
        .await
        .map_err(|e| format!("receive_deactivated: {e}"))?;

    while running.load(Ordering::SeqCst) {
        tokio::select! {
            ev = activated.next() => {
                match ev {
                    Some(a) if a.shortcut_id() == SHORTCUT_ID => {
                        let _ = tx.send(HotkeyEvent::Pressed);
                    }
                    Some(_) => {}
                    None => break,
                }
            }
            ev = deactivated.next() => {
                match ev {
                    Some(d) if d.shortcut_id() == SHORTCUT_ID => {
                        let _ = tx.send(HotkeyEvent::Released);
                    }
                    Some(_) => {}
                    None => break,
                }
            }
        }
    }

    Ok(())
}

/// Hold backend selection hint (mirrors `lira_core::HoldBackend` without a core dep).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoldBackendHint {
    Compositor,
    Portal,
    Evdev,
}

/// Resolve the hold backend that should be attempted.
/// Never auto-selects Evdev.
pub fn recommend_backend(configured: HoldBackendHint, session_is_niri: bool) -> HoldBackendHint {
    match configured {
        HoldBackendHint::Evdev => HoldBackendHint::Evdev,
        HoldBackendHint::Portal => HoldBackendHint::Portal,
        HoldBackendHint::Compositor => {
            if session_is_niri {
                HoldBackendHint::Compositor
            } else {
                // Off Niri, portal is the preferred hold path when available.
                HoldBackendHint::Portal
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recommend_keeps_niri_on_compositor() {
        assert_eq!(
            recommend_backend(HoldBackendHint::Compositor, true),
            HoldBackendHint::Compositor
        );
    }

    #[test]
    fn recommend_prefers_portal_off_niri_when_compositor_default() {
        assert_eq!(
            recommend_backend(HoldBackendHint::Compositor, false),
            HoldBackendHint::Portal
        );
    }
}
