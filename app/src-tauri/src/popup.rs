use std::sync::Arc;

use lira_core::settings::{PopupMode, Settings};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::AppRuntime;

pub fn show_popup(app: &AppHandle, runtime: &Arc<AppRuntime>) {
    if app.get_webview_window("popup").is_some() {
        return;
    }
    let settings = runtime.settings.lock().expect("settings").clone();
    let use_layer = should_use_layer_shell(&settings);

    let url = WebviewUrl::App("index.html".into());
    let builder = WebviewWindowBuilder::new(app, "popup", url)
        .title("Lira")
        .inner_size(360.0, 280.0)
        .resizable(false);

    let window = if use_layer {
        builder.decorations(false).visible(false).build()
    } else {
        builder.decorations(true).visible(true).build()
    };

    match window {
        Ok(win) if use_layer => {
            let app_main = app.clone();
            let runtime_main = Arc::clone(runtime);
            let win_for_main = win.clone();
            if let Err(err) = app.run_on_main_thread(move || {
                if !apply_layer_shell(&win_for_main) {
                    log::warn!("layer-shell setup failed; showing floating popup");
                    let _ = win_for_main.set_decorations(true);
                }
                let _ = win_for_main.show();
                crate::emit_popup_status(&app_main, &runtime_main);
            }) {
                log::warn!("run_on_main_thread for popup failed: {err}");
                let _ = win.set_decorations(true);
                let _ = win.show();
            }
        }
        Ok(win) => {
            let _ = win.show();
            crate::emit_popup_status(app, runtime);
        }
        Err(err) => log::error!("failed to open popup: {err}"),
    }
}

pub fn hide_popup(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("popup") {
        let _ = window.close();
    }
}

fn should_use_layer_shell(settings: &Settings) -> bool {
    #[cfg(not(target_os = "linux"))]
    {
        return false;
    }
    #[cfg(target_os = "linux")]
    match settings.ui.popup {
        PopupMode::Floating => false,
        PopupMode::LayerShell => true,
        PopupMode::Auto => gtk_layer_shell_is_supported(),
    }
}

#[cfg(target_os = "linux")]
fn gtk_layer_shell_is_supported() -> bool {
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
