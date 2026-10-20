pub mod niri;

use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionKind {
    Niri,
    Hyprland,
    Kde,
    Gnome,
    X11,
    Unknown,
}

pub fn detect_session() -> SessionKind {
    let session_type = env::var("XDG_SESSION_TYPE")
        .unwrap_or_default()
        .to_lowercase();
    let desktop = env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_default()
        .to_lowercase();

    if env::var_os("NIRI_SOCKET").is_some() || desktop.contains("niri") {
        return SessionKind::Niri;
    }
    if env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some() || desktop.contains("hyprland") {
        return SessionKind::Hyprland;
    }
    if desktop.contains("kde") {
        return SessionKind::Kde;
    }
    if desktop.contains("gnome") {
        return SessionKind::Gnome;
    }
    if session_type == "x11" {
        return SessionKind::X11;
    }
    SessionKind::Unknown
}
