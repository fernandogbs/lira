use std::env;

use crate::settings::{ScreenContext, UiLocale};
use crate::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    PtBr,
    En,
}

impl UiLocale {
    pub fn resolve(self) -> Locale {
        match self {
            Self::System => Locale::detect(),
            Self::PtBr => Locale::PtBr,
            Self::En => Locale::En,
        }
    }
}

impl Locale {
    pub fn detect() -> Self {
        for key in ["LC_ALL", "LC_MESSAGES", "LANG"] {
            if let Ok(val) = env::var(key) {
                let lower = val.to_lowercase();
                if lower.starts_with("pt") {
                    return Self::PtBr;
                }
                if lower.starts_with("en") {
                    return Self::En;
                }
            }
        }
        Self::En
    }

    pub fn tag(self) -> &'static str {
        match self {
            Self::PtBr => "pt-BR",
            Self::En => "en",
        }
    }

    pub fn catalog(self) -> &'static Catalog {
        match self {
            Self::PtBr => &CATALOG_PT_BR,
            Self::En => &CATALOG_EN,
        }
    }

    pub fn not_running(self) -> &'static str {
        self.catalog().not_running
    }

    pub fn status_line(self, state: AppState) -> String {
        match state {
            AppState::Idle => self.catalog().state_idle.to_string(),
            AppState::Listening => self.catalog().state_listening.to_string(),
            AppState::Transcribing => self.catalog().state_transcribing.to_string(),
        }
    }

    pub fn command_ok(self, cmd: &str) -> String {
        match self {
            Self::PtBr => format!("Comando '{}' aceito", cmd),
            Self::En => format!("Command '{}' accepted", cmd),
        }
    }

    pub fn hotkey_mode_label(self, mode: crate::settings::HotkeyMode) -> String {
        let c = self.catalog();
        match mode {
            crate::settings::HotkeyMode::Toggle => c.hotkey_toggle.to_string(),
            crate::settings::HotkeyMode::Hold => c.hotkey_hold.to_string(),
        }
    }

    pub fn screen_context_label(self, context: ScreenContext) -> String {
        let c = self.catalog();
        match context {
            ScreenContext::Off => c.screen_context_off.to_string(),
            ScreenContext::Text => c.screen_context_text.to_string(),
            ScreenContext::Image => c.screen_context_image.to_string(),
        }
    }

    pub fn screen_context_note(self, context: ScreenContext) -> Option<String> {
        let c = self.catalog();
        match context {
            ScreenContext::Off => None,
            ScreenContext::Text => Some(c.screen_context_text_hint.to_string()),
            ScreenContext::Image => Some(c.screen_context_image_warning.to_string()),
        }
    }
}

pub struct Catalog {
    pub not_running: &'static str,
    pub state_idle: &'static str,
    pub state_listening: &'static str,
    pub state_transcribing: &'static str,
    pub hotkey_toggle: &'static str,
    pub hotkey_hold: &'static str,
    pub tray_toggle: &'static str,
    pub tray_quit: &'static str,
    pub screen_context_off: &'static str,
    pub screen_context_text: &'static str,
    pub screen_context_image: &'static str,
    pub screen_context_text_hint: &'static str,
    pub screen_context_image_warning: &'static str,
}

const CATALOG_PT_BR: Catalog = Catalog {
    not_running: "Lira não está em execução",
    state_idle: "Ocioso",
    state_listening: "Ouvindo…",
    state_transcribing: "Transcrevendo…",
    hotkey_toggle: "alternar (toggle)",
    hotkey_hold: "segurar (hold)",
    tray_toggle: "Alternar",
    tray_quit: "Sair",
    screen_context_off: "desligado",
    screen_context_text: "texto",
    screen_context_image: "imagem",
    screen_context_text_hint: "Modo texto: OCR local quando implementado (recomendado).",
    screen_context_image_warning: "Modo imagem: envia capturas ao modelo — maior custo em tokens.",
};

const CATALOG_EN: Catalog = Catalog {
    not_running: "Lira is not running",
    state_idle: "Idle",
    state_listening: "Listening…",
    state_transcribing: "Transcribing…",
    hotkey_toggle: "toggle",
    hotkey_hold: "hold",
    tray_toggle: "Toggle",
    tray_quit: "Quit",
    screen_context_off: "off",
    screen_context_text: "text",
    screen_context_image: "image",
    screen_context_text_hint: "Text mode: local OCR when implemented (recommended).",
    screen_context_image_warning: "Image mode: sends screenshots to the model — higher token cost.",
};

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    fn catalog_entries(locale: Locale) -> BTreeMap<&'static str, &'static str> {
        let c = locale.catalog();
        BTreeMap::from([
            ("not_running", c.not_running),
            ("state_idle", c.state_idle),
            ("state_listening", c.state_listening),
            ("state_transcribing", c.state_transcribing),
            ("hotkey_toggle", c.hotkey_toggle),
            ("hotkey_hold", c.hotkey_hold),
            ("tray_toggle", c.tray_toggle),
            ("tray_quit", c.tray_quit),
            ("screen_context_off", c.screen_context_off),
            ("screen_context_text", c.screen_context_text),
            ("screen_context_image", c.screen_context_image),
            ("screen_context_text_hint", c.screen_context_text_hint),
            (
                "screen_context_image_warning",
                c.screen_context_image_warning,
            ),
        ])
    }

    #[test]
    fn locale_catalog_keys_match() {
        let pt = catalog_entries(Locale::PtBr);
        let en = catalog_entries(Locale::En);
        assert_eq!(pt.keys().collect::<Vec<_>>(), en.keys().collect::<Vec<_>>());
        for key in pt.keys() {
            assert!(!pt[key].is_empty());
            assert!(!en[key].is_empty());
        }
    }
}
