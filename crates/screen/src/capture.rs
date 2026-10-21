//! Capture backends: Niri IPC, portal Screenshot, X11 via grim/import fallbacks.

use std::path::PathBuf;
use std::process::Command;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CaptureError {
    #[error("{0}")]
    Msg(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub struct FocusedWindow {
    pub id: u64,
    pub app_id: String,
    pub title: String,
}

#[derive(Debug, Clone)]
pub struct CaptureResult {
    pub png_bytes: Vec<u8>,
    pub focused: Option<FocusedWindow>,
}

/// Capture the focused window (Niri) or a portal/full-screen fallback.
/// PNG bytes stay in memory; any temp file is deleted immediately.
pub fn capture_for_context(scope_window: bool) -> Result<CaptureResult, CaptureError> {
    if std::env::var_os("NIRI_SOCKET").is_some() {
        return capture_niri(scope_window);
    }
    if std::env::var("XDG_SESSION_TYPE")
        .map(|s| s.eq_ignore_ascii_case("x11"))
        .unwrap_or(false)
    {
        return capture_x11();
    }
    capture_portal()
}

fn capture_niri(scope_window: bool) -> Result<CaptureResult, CaptureError> {
    let focused = niri_focused_window()?;
    let tmp = runtime_tmp_png()?;
    let mut cmd = Command::new("niri");
    cmd.args(["msg", "action", "screenshot-window"]);
    if let Some(ref fw) = focused {
        cmd.arg("--id").arg(fw.id.to_string());
    }
    cmd.arg("--path").arg(&tmp);
    let status = cmd.status().map_err(|e| CaptureError::Msg(e.to_string()))?;
    if !status.success() && scope_window {
        // Fall back to screen screenshot action
        let _ = Command::new("niri")
            .args(["msg", "action", "screenshot-screen", "--path"])
            .arg(&tmp)
            .status();
    }
    let png = read_and_remove(&tmp)?;
    if png.is_empty() {
        return Err(CaptureError::Msg(
            "niri screenshot produced empty file".into(),
        ));
    }
    Ok(CaptureResult {
        png_bytes: png,
        focused,
    })
}

fn niri_focused_window() -> Result<Option<FocusedWindow>, CaptureError> {
    let output = Command::new("niri")
        .args(["msg", "--json", "focused-window"])
        .output()
        .map_err(|e| CaptureError::Msg(format!("niri msg: {e}")))?;
    if !output.status.success() {
        return Ok(None);
    }
    let v: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| CaptureError::Msg(format!("parse focused-window: {e}")))?;
    let id = v.get("id").and_then(|x| x.as_u64()).unwrap_or(0);
    let app_id = v
        .get("app_id")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    let title = v
        .get("title")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    if id == 0 && app_id.is_empty() {
        return Ok(None);
    }
    Ok(Some(FocusedWindow { id, app_id, title }))
}

fn capture_portal() -> Result<CaptureResult, CaptureError> {
    // Portable CLI fallback used by many desktops: gnome-screenshot / spectacle / grim.
    let tmp = runtime_tmp_png()?;
    let attempts = [
        ("grim", vec![tmp.to_string_lossy().to_string()]),
        (
            "gnome-screenshot",
            vec!["-f".into(), tmp.to_string_lossy().to_string()],
        ),
        (
            "spectacle",
            vec![
                "-b".into(),
                "-n".into(),
                "-o".into(),
                tmp.to_string_lossy().to_string(),
            ],
        ),
    ];
    for (bin, args) in attempts {
        if Command::new(bin)
            .args(&args)
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            let png = read_and_remove(&tmp)?;
            if !png.is_empty() {
                return Ok(CaptureResult {
                    png_bytes: png,
                    focused: None,
                });
            }
        }
    }
    Err(CaptureError::Msg(
        "no screenshot tool succeeded (grim/gnome-screenshot/spectacle)".into(),
    ))
}

fn capture_x11() -> Result<CaptureResult, CaptureError> {
    let tmp = runtime_tmp_png()?;
    let ok = Command::new("import")
        .args(["-window", "root", tmp.to_str().unwrap_or("/tmp/lira.png")])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !ok {
        return capture_portal();
    }
    let png = read_and_remove(&tmp)?;
    Ok(CaptureResult {
        png_bytes: png,
        focused: None,
    })
}

fn runtime_tmp_png() -> Result<PathBuf, CaptureError> {
    let dir = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let path = dir.join(format!("lira-screen-{}.png", std::process::id()));
    Ok(path)
}

fn read_and_remove(path: &PathBuf) -> Result<Vec<u8>, CaptureError> {
    let bytes = std::fs::read(path).unwrap_or_default();
    let _ = std::fs::remove_file(path);
    Ok(bytes)
}
