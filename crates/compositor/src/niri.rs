//! Niri helpers via `niri msg` JSON (no `niri-ipc` crate — GPL).

use std::process::Command;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct FocusedWindow {
    pub id: u64,
    #[serde(default)]
    pub app_id: String,
    #[serde(default)]
    pub title: String,
}

/// Query the focused window over the Niri CLI.
pub fn focused_window() -> Option<FocusedWindow> {
    let output = Command::new("niri")
        .args(["msg", "--json", "focused-window"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    serde_json::from_slice(&output.stdout).ok()
}

#[cfg(test)]
mod tests {
    #[test]
    fn stub_compiles() {
        // focused_window() needs a live Niri session; just ensure types link.
        let _ = super::focused_window();
    }
}
