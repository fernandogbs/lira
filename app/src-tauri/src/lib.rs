mod popup;
mod voice;

use std::sync::{Arc, Mutex};

use lira_core::ipc::{Command, CommandHandler, IpcServer, Response};
use lira_core::settings::{HotkeyMode, ScreenContext, Settings};
use lira_core::{AppState, Locale};
use serde::Serialize;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, State};
use voice::VoiceHandle;

pub struct AppRuntime {
    state: Mutex<AppState>,
    settings: Mutex<Settings>,
    locale: Locale,
}

#[derive(Clone, Serialize)]
struct PopupPayload {
    state: AppState,
    hotkey_mode: HotkeyMode,
    screen_context: ScreenContext,
    locale_tag: String,
    state_label: String,
    hotkey_mode_label: String,
    screen_context_label: String,
    screen_context_note: Option<String>,
}

fn popup_payload(runtime: &AppRuntime) -> PopupPayload {
    let state = *runtime.state.lock().expect("state lock");
    let settings = runtime.settings.lock().expect("settings lock");
    let locale = runtime.locale;
    PopupPayload {
        state,
        hotkey_mode: settings.hotkey_mode(),
        screen_context: settings.screen.context,
        locale_tag: locale.tag().to_string(),
        state_label: locale.status_line(state),
        hotkey_mode_label: locale.hotkey_mode_label(settings.hotkey_mode()),
        screen_context_label: locale.screen_context_label(settings.screen.context),
        screen_context_note: locale.screen_context_note(settings.screen.context),
    }
}

pub fn emit_popup_status(app: &AppHandle, runtime: &Arc<AppRuntime>) {
    let payload = popup_payload(runtime);
    let _ = app.emit("lira://status", payload);
}

#[tauri::command]
fn get_popup_status(runtime: State<'_, Arc<AppRuntime>>) -> PopupPayload {
    popup_payload(&runtime)
}

fn apply_command(
    app: &AppHandle,
    runtime: &Arc<AppRuntime>,
    voice: &VoiceHandle,
    command: Command,
) -> Response {
    match command {
        Command::Toggle => voice::handle_toggle(app, runtime, voice),
        Command::Start => voice::handle_start(app, runtime, voice),
        Command::Stop => voice::handle_stop(app, runtime, voice),
        Command::Status => {}
    }
    let current = *runtime.state.lock().expect("state lock");
    emit_popup_status(app, runtime);
    Response::ok(current)
}

fn tray_disabled() -> bool {
    std::env::var("LIRA_NO_TRAY")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

#[cfg(target_os = "linux")]
fn status_notifier_watcher_present() -> bool {
    let output = std::process::Command::new("dbus-send")
        .args([
            "--session",
            "--print-reply",
            "--dest=org.freedesktop.DBus",
            "/org/freedesktop/DBus",
            "org.freedesktop.DBus.NameHasOwner",
            "string:org.kde.StatusNotifierWatcher",
        ])
        .output();
    match output {
        Ok(out) if out.status.success() => {
            let text = String::from_utf8_lossy(&out.stdout);
            text.contains("boolean true")
        }
        Ok(out) => {
            log::debug!(
                "dbus-send NameHasOwner failed: {}",
                String::from_utf8_lossy(&out.stderr)
            );
            false
        }
        Err(err) => {
            log::debug!("dbus-send unavailable: {err}");
            false
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn status_notifier_watcher_present() -> bool {
    false
}

fn setup_tray(app: &AppHandle, locale: Locale) -> Result<(), Box<dyn std::error::Error>> {
    if tray_disabled() {
        log::info!("tray disabled via LIRA_NO_TRAY");
        return Ok(());
    }
    if !status_notifier_watcher_present() {
        log::warn!(
            "tray: org.kde.StatusNotifierWatcher not found on session bus; running without tray icon (use `lira toggle` or the compositor keybind). Enable a StatusNotifier host (e.g. DMS tray widget, waybar tray) to get the icon."
        );
        return Ok(());
    }
    log::info!("tray: StatusNotifierWatcher found, tray enabled");

    let catalog = locale.catalog();
    let toggle = MenuItem::with_id(app, "toggle", catalog.tray_toggle, true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", catalog.tray_quit, true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&toggle, &quit])?;
    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().cloned().unwrap())
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "toggle" => {
                if let Some(runtime) = app.try_state::<Arc<AppRuntime>>() {
                    if let Some(voice) = app.try_state::<VoiceHandle>() {
                        apply_command(app, &runtime, &voice, Command::Toggle);
                    }
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(runtime) = app.try_state::<Arc<AppRuntime>>() {
                    if let Some(voice) = app.try_state::<VoiceHandle>() {
                        apply_command(app, &runtime, &voice, Command::Toggle);
                    }
                }
            }
        })
        .build(app)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let settings = Settings::load().unwrap_or_default();
    if settings.hotkey_mode() == HotkeyMode::Hold {
        log::warn!("hotkey mode=hold is accepted but not implemented in M0 (stub)");
    }

    let locale = settings.ui.locale.resolve();

    let runtime = Arc::new(AppRuntime {
        state: Mutex::new(AppState::Idle),
        settings: Mutex::new(settings),
        locale,
    });

    let session = lira_compositor::detect_session();
    log::info!("detected session: {:?}", session);

    tauri::Builder::default()
        .manage(Arc::clone(&runtime))
        .invoke_handler(tauri::generate_handler![get_popup_status])
        .setup(move |app| {
            let voice = VoiceHandle::spawn(app.handle().clone(), Arc::clone(&runtime));
            app.manage(voice);

            let tray_locale = runtime.locale;
            if let Err(err) = setup_tray(app.handle(), tray_locale) {
                log::warn!("tray setup failed (ignored): {err}");
            }

            let handle = app.handle().clone();
            let rt = Arc::clone(&runtime);
            let handler: CommandHandler = Arc::new(move |cmd| {
                let voice = handle.state::<VoiceHandle>();
                apply_command(&handle, &rt, &voice, cmd)
            });

            let socket_path = lira_core::ipc::socket_path();
            tauri::async_runtime::spawn(async move {
                match IpcServer::bind(&socket_path).await {
                    Ok(server) => {
                        log::info!("IPC listening on {}", server.path().display());
                        if let Err(err) = server.run(handler).await {
                            log::error!("IPC server error: {err}");
                        }
                    }
                    Err(err) => log::error!("failed to bind IPC socket: {err}"),
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
