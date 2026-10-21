//! Popup window. Every function that touches the window or GTK runs on the
//! GTK main thread via `main_thread::run_on_main`; callers may be on any thread
//! (tokio IPC workers, voice threads, tray events on the main thread).

use std::sync::Arc;

use lira_core::settings::{PopupMode, Settings};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::main_thread::{assert_gtk_main_thread, run_on_main};
use crate::AppRuntime;

/// How the popup ended up being shown (for logs).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupShown {
    AlreadyOpen,
    LayerShell,
    Floating,
}

pub fn show_popup(app: &AppHandle, runtime: &Arc<AppRuntime>) {
    let settings = runtime.settings.lock().expect("settings").clone();
    let app_main = app.clone();
    let runtime_main = Arc::clone(runtime);
    match run_on_main(app, move || {
        show_popup_on_main(&app_main, &runtime_main, &settings)
    }) {
        Ok(Ok(PopupShown::AlreadyOpen)) => {}
        Ok(Ok(mode)) => log::info!("popup shown ({mode:?})"),
        Ok(Err(err)) => log::error!("failed to open popup: {err}"),
        Err(err) => log::error!("show_popup: {err}"),
    }
}

pub fn hide_popup(app: &AppHandle) {
    let app_main = app.clone();
    let result = run_on_main(app, move || {
        assert_gtk_main_thread();
        match app_main.get_webview_window("popup") {
            Some(window) => window.close().map_err(|e| e.to_string()),
            None => Ok(()),
        }
    });
    match result {
        Ok(Ok(())) => {}
        Ok(Err(err)) => log::warn!("failed to close popup: {err}"),
        Err(err) => log::error!("hide_popup: {err}"),
    }
}

fn show_popup_on_main(
    app: &AppHandle,
    runtime: &Arc<AppRuntime>,
    settings: &Settings,
) -> Result<PopupShown, String> {
    assert_gtk_main_thread();
    if app.get_webview_window("popup").is_some() {
        return Ok(PopupShown::AlreadyOpen);
    }
    let use_layer = should_use_layer_shell(settings);

    let url = WebviewUrl::App("index.html".into());
    let win = WebviewWindowBuilder::new(app, "popup", url)
        .title("Lira")
        .inner_size(360.0, 280.0)
        .resizable(false)
        .decorations(!use_layer)
        .visible(false)
        .build()
        .map_err(|e| e.to_string())?;

    let mode = if use_layer && apply_layer_shell(&win) {
        PopupShown::LayerShell
    } else {
        if use_layer {
            log::warn!("layer-shell setup failed; showing floating popup");
            let _ = win.set_decorations(true);
        }
        PopupShown::Floating
    };
    win.show().map_err(|e| e.to_string())?;
    crate::emit_popup_status(app, runtime);
    Ok(mode)
}

fn should_use_layer_shell(settings: &Settings) -> bool {
    match settings.ui.popup {
        PopupMode::Floating => false,
        PopupMode::LayerShell => cfg!(target_os = "linux"),
        PopupMode::Auto => gtk_layer_shell_is_supported(),
    }
}

#[cfg(target_os = "linux")]
fn gtk_layer_shell_is_supported() -> bool {
    assert_gtk_main_thread();
    gtk_layer_shell::is_supported()
}

#[cfg(not(target_os = "linux"))]
fn gtk_layer_shell_is_supported() -> bool {
    false
}

#[cfg(target_os = "linux")]
fn apply_layer_shell(window: &tauri::WebviewWindow) -> bool {
    use gtk::prelude::WidgetExt;
    use gtk_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

    assert_gtk_main_thread();
    let gtk_win = match window.gtk_window() {
        Ok(w) => w,
        Err(err) => {
            log::warn!("gtk_window unavailable: {err}");
            return false;
        }
    };
    if gtk_win.is_realized() {
        gtk_win.unrealize();
    }
    gtk_win.init_layer_shell();
    if !gtk_win.is_layer_window() {
        return false;
    }
    gtk_win.set_layer(Layer::Overlay);
    gtk_win.set_namespace("lira");
    gtk_win.set_keyboard_mode(KeyboardMode::OnDemand);
    gtk_win.set_anchor(Edge::Top, true);
    gtk_win.set_anchor(Edge::Right, true);
    gtk_win.set_layer_shell_margin(Edge::Top, 12);
    gtk_win.set_layer_shell_margin(Edge::Right, 12);
    true
}

#[cfg(not(target_os = "linux"))]
fn apply_layer_shell(_window: &tauri::WebviewWindow) -> bool {
    false
}
