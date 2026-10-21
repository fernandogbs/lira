//! Default app_id exclusion list (password managers, banks).

/// Return true if the focused app must not be captured.
pub fn is_excluded(app_id: &str, extra: &[String]) -> bool {
    let id = app_id.to_lowercase();
    DEFAULT_EXCLUDES.iter().any(|e| id.contains(e))
        || extra.iter().any(|e| id.contains(&e.to_lowercase()))
}

const DEFAULT_EXCLUDES: &[&str] = &[
    "1password",
    "bitwarden",
    "keepassxc",
    "keepass",
    "org.gnome.seahorse",
    "keychain",
    "itau",
    "bradesco",
    "nubank",
    "inter",
    "caixa",
    "bb.com",
    "bancodobrasil",
    "santander",
];

/// Suggested Niri window-rule snippet for block-out-from screen-capture.
pub fn niri_block_out_snippet() -> String {
    let mut out = String::from("// Add to ~/.config/niri/config.kdl\n");
    for app in ["1Password", "bitwarden", "org.keepassxc.KeePassXC"] {
        out.push_str(&format!(
            "window-rule {{\n    match app-id=\"^{app}$\"\n    block-out-from \"screen-capture\"\n}}\n"
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn excludes_bitwarden() {
        assert!(is_excluded("com.bitwarden.desktop", &[]));
    }

    #[test]
    fn allows_editor() {
        assert!(!is_excluded("code", &[]));
    }
}
