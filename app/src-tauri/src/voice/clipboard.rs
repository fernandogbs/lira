pub fn copy_to_clipboard(text: &str) {
    if text.is_empty() {
        return;
    }
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        if copy_wl_copy(text) {
            return;
        }
        log::warn!("wl-copy failed; trying arboard");
    }
    if let Err(err) = copy_arboard(text) {
        log::warn!("clipboard copy failed: {err}");
    }
}

fn copy_wl_copy(text: &str) -> bool {
    let mut child = match std::process::Command::new("wl-copy")
        .stdin(std::process::Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(err) => {
            log::warn!("failed to spawn wl-copy: {err}");
            return false;
        }
    };
    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        if let Err(err) = stdin.write_all(text.as_bytes()) {
            log::warn!("wl-copy stdin write failed: {err}");
            return false;
        }
    }
    match child.wait() {
        Ok(status) if status.success() => true,
        Ok(status) => {
            log::warn!("wl-copy exited with {status}");
            false
        }
        Err(err) => {
            log::warn!("wl-copy wait failed: {err}");
            false
        }
    }
}

fn copy_arboard(text: &str) -> Result<(), arboard::Error> {
    let mut clipboard = arboard::Clipboard::new()?;
    clipboard.set_text(text)?;
    Ok(())
}
