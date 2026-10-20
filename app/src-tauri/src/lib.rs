use std::sync::{Arc, Mutex};

use lira_core::ipc::{Command, CommandHandler, IpcServer, Response};
use lira_core::settings::{HotkeyMode, Settings};
use lira_core::{AppState, Locale};
use serde::Serialize;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder};

struct AppRuntime {
    state: Mutex<AppState>,
    settings: Settings,
    locale: Locale,
}

#[derive(Clone, Serialize)]
struct PopupPayload {
    state: AppState,
    hotkey_mode: HotkeyMode,
    state_label: String,
    hotkey_mode_label: String,
}

#[tauri::command]
fn get_popup_status(runtime: State<'_, Arc<AppRuntime>>) -> PopupPayload {
    let state = *runtime.state.lock().expect("state lock");
    let locale = runtime.locale;
    PopupPayload {
        state,
        hotkey_mode: runtime.settings.hotkey_mode,
        state_label: locale.status_line(state),
        hotkey_mode_label: hotkey_mode_label(locale, runtime.settings.hotkey_mode),
    }
}

fn hotkey_mode_label(locale: Locale, mode: HotkeyMode) -> String {
    match (locale, mode) {
        (Locale::PtBr, HotkeyMode::Toggle) => "alternar (toggle)".into(),
        (Locale::PtBr, HotkeyMode::Hold) => "segurar (hold, stub)".into(),
        (Locale::En, HotkeyMode::Toggle) => "toggle".into(),
        (Locale::En, HotkeyMode::Hold) => "hold (stub)".into(),
    }
}

fn emit_popup_status(app: &AppHandle, runtime: &AppRuntime) {
    let state = *runtime.state.lock().expect("state lock");
    let payload = PopupPayload {
        state,
        hotkey_mode: runtime.settings.hotkey_mode,
        state_label: runtime.locale.status_line(state),
        hotkey_mode_label: hotkey_mode_label(runtime.locale, runtime.settings.hotkey_mode),
    };
    let _ = app.emit("lira://status", payload);
}

fn show_popup(app: &AppHandle) {
    if app.get_webview_window("popup").is_some() {
        return;
    }
    let url = WebviewUrl::App("index.html".into());
    if let Err(err) = WebviewWindowBuilder::new(app, "popup", url)
        .title("Lira")
        .inner_size(320.0, 140.0)
        .resizable(false)
        .decorations(true)
        .visible(true)
        .build()
    {
        log::error!("failed to open popup: {err}");
    }
}

fn hide_popup(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("popup") {
        let _ = window.close();
    }
}

fn apply_command(app: &AppHandle, runtime: &Arc<AppRuntime>, command: Command) -> Response {
    let mut state = runtime.state.lock().expect("state lock");
    match command {
        Command::Toggle => {
            if app.get_webview_window("popup").is_some() {
                hide_popup(app);
                *state = AppState::Idle;
            } else {
                *state = state.toggle();
                show_popup(app);
            }
        }
        Command::Start => {
            *state = AppState::Listening;
            show_popup(app);
        }
        Command::Stop => {
            *state = AppState::Idle;
            hide_popup(app);
        }
        Command::Status => {}
    }
    let current = *state;
    emit_popup_status(app, runtime);
    Response::ok(current)
}

fn tray_disabled() -> bool {
    std::env::var("LIRA_NO_TRAY")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    if tray_disabled() {
        log::info!("tray disabled via LIRA_NO_TRAY");
        return Ok(());
    }
    let toggle = MenuItem::with_id(app, "toggle", "Toggle", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&toggle, &quit])?;
    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().cloned().unwrap())
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "toggle" => {
                if let Some(runtime) = app.try_state::<Arc<AppRuntime>>() {
                    apply_command(app, &runtime, Command::Toggle);
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
                    apply_command(app, &runtime, Command::Toggle);
                }
            }
        })
        .build(app)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let settings = Settings::load().unwrap_or_default();
    if settings.hotkey_mode == HotkeyMode::Hold {
        log::warn!("hotkey_mode=hold is accepted but not implemented in M0 (stub)");
    }

    let runtime = Arc::new(AppRuntime {
        state: Mutex::new(AppState::Idle),
        settings,
        locale: Locale::detect(),
    });

    let session = lira_compositor::detect_session();
    log::info!("detected session: {:?}", session);

    tauri::Builder::default()
        .manage(Arc::clone(&runtime))
        .invoke_handler(tauri::generate_handler![get_popup_status])
        .setup(move |app| {
            if let Err(err) = setup_tray(app.handle()) {
                log::warn!("tray setup failed (ignored): {err}");
            }

            let handle = app.handle().clone();
            let rt = Arc::clone(&runtime);
            let handler: CommandHandler = Arc::new(move |cmd| apply_command(&handle, &rt, cmd));

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
