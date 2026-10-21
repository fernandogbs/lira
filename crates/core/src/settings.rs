use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

const INITIAL_PROMPT_MAX_CHARS: usize = 200;
const VAD_AUTOSTOP_MAX_MS: u32 = 5000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HotkeyMode {
    #[default]
    Toggle,
    Hold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UiLocale {
    #[default]
    System,
    #[serde(rename = "pt-BR")]
    PtBr,
    En,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScreenContext {
    #[default]
    Off,
    Text,
    Image,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SttModel {
    #[default]
    #[serde(rename = "large-v3-turbo-q5_0")]
    LargeV3TurboQ5_0,
    #[serde(rename = "small-q5_1")]
    SmallQ5_1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SttLanguage {
    #[default]
    Auto,
    Pt,
    En,
}

fn default_vad_autostop_ms() -> u32 {
    1500
}

fn default_stt_engine() -> String {
    "whisper".to_string()
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeySection {
    #[serde(default)]
    pub mode: HotkeyMode,
    #[serde(default = "default_vad_autostop_ms")]
    pub vad_autostop_ms: u32,
}

impl Default for HotkeySection {
    fn default() -> Self {
        Self {
            mode: HotkeyMode::default(),
            vad_autostop_ms: default_vad_autostop_ms(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiSection {
    #[serde(default)]
    pub locale: UiLocale,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScreenSection {
    #[serde(default)]
    pub context: ScreenContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SttSection {
    #[serde(default = "default_stt_engine")]
    pub engine: String,
    #[serde(default)]
    pub model: SttModel,
    #[serde(default)]
    pub language: SttLanguage,
    #[serde(default)]
    pub vocabulary: Vec<String>,
    #[serde(default)]
    pub gpu: bool,
    #[serde(default = "default_fallback_model")]
    pub fallback_model: SttModel,
}

fn default_fallback_model() -> SttModel {
    SttModel::SmallQ5_1
}

impl Default for SttSection {
    fn default() -> Self {
        Self {
            engine: default_stt_engine(),
            model: SttModel::default(),
            language: SttLanguage::default(),
            vocabulary: Vec::new(),
            gpu: false,
            fallback_model: default_fallback_model(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistorySection {
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl Default for HistorySection {
    fn default() -> Self {
        Self {
            enabled: default_true(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub hotkey: HotkeySection,
    #[serde(default)]
    pub ui: UiSection,
    #[serde(default)]
    pub screen: ScreenSection,
    #[serde(default)]
    pub stt: SttSection,
    #[serde(default)]
    pub history: HistorySection,
}

impl UiLocale {
    /// STT language code (`pt` or `en`) when UI locale drives transcription.
    pub fn stt_language_code(self) -> &'static str {
        match self {
            Self::PtBr => "pt",
            Self::En => "en",
            Self::System => detect_system_language_code(),
        }
    }
}

fn detect_system_language_code() -> &'static str {
    for key in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(val) = std::env::var(key) {
            let lower = val.to_lowercase();
            if lower.starts_with("pt") {
                return "pt";
            }
            if lower.starts_with("en") {
                return "en";
            }
        }
    }
    "en"
}

impl Settings {
    pub fn hotkey_mode(&self) -> HotkeyMode {
        self.hotkey.mode
    }

    pub fn effective_language(&self) -> &'static str {
        match self.stt.language {
            SttLanguage::Pt => "pt",
            SttLanguage::En => "en",
            SttLanguage::Auto => self.ui.locale.stt_language_code(),
        }
    }

    pub fn initial_prompt(&self) -> Option<String> {
        if self.stt.vocabulary.is_empty() {
            return None;
        }
        let joined = self.stt.vocabulary.join(", ");
        if joined.len() <= INITIAL_PROMPT_MAX_CHARS {
            return Some(joined);
        }
        let mut out = joined;
        out.truncate(INITIAL_PROMPT_MAX_CHARS);
        while !out.is_empty() && !out.is_char_boundary(out.len()) {
            out.pop();
        }
        Some(out)
    }

    fn normalize(&mut self) {
        self.hotkey.vad_autostop_ms = self.hotkey.vad_autostop_ms.min(VAD_AUTOSTOP_MAX_MS);
    }
}

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("failed to read config: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse config: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("failed to serialize config: {0}")]
    Serialize(#[from] toml::ser::Error),
}

pub fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("lira").join("config.toml"))
}

impl Settings {
    pub fn load() -> Result<Self, SettingsError> {
        let path = config_path().ok_or_else(|| {
            SettingsError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "no config directory",
            ))
        })?;
        if !path.exists() {
            let settings = Self::default();
            settings.save()?;
            return Ok(settings);
        }
        let text = fs::read_to_string(&path)?;
        let mut settings: Self = toml::from_str(&text)?;
        settings.normalize();
        Ok(settings)
    }

    pub fn save(&self) -> Result<(), SettingsError> {
        let path = config_path().ok_or_else(|| {
            SettingsError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "no config directory",
            ))
        })?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let text = toml::to_string_pretty(self)?;
        fs::write(path, text)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::Mutex;
    use tempfile::tempdir;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn parses_sectioned_config() {
        let _env_guard = ENV_LOCK.lock().unwrap();
        let dir = tempdir().unwrap();
        let config_dir = dir.path().join("lira");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(
            config_dir.join("config.toml"),
            r#"
[hotkey]
mode = "hold"

[ui]
locale = "pt-BR"

[screen]
context = "text"

[future]
enabled = true
"#,
        )
        .unwrap();

        let prev = env::var("XDG_CONFIG_HOME").ok();
        env::set_var("XDG_CONFIG_HOME", dir.path());

        let settings = Settings::load().unwrap();
        assert_eq!(settings.hotkey.mode, HotkeyMode::Hold);
        assert_eq!(settings.ui.locale, UiLocale::PtBr);
        assert_eq!(settings.screen.context, ScreenContext::Text);

        if let Some(p) = prev {
            env::set_var("XDG_CONFIG_HOME", p);
        } else {
            env::remove_var("XDG_CONFIG_HOME");
        }
    }

    #[test]
    fn default_config_round_trip() {
        let settings = Settings::default();
        let text = toml::to_string_pretty(&settings).unwrap();
        let parsed: Settings = toml::from_str(&text).unwrap();
        assert_eq!(parsed.hotkey.mode, HotkeyMode::Toggle);
        assert_eq!(parsed.hotkey.vad_autostop_ms, 1500);
        assert_eq!(parsed.ui.locale, UiLocale::System);
        assert_eq!(parsed.screen.context, ScreenContext::Off);
        assert_eq!(parsed.stt.engine, "whisper");
        assert_eq!(parsed.stt.model, SttModel::LargeV3TurboQ5_0);
        assert_eq!(parsed.stt.language, SttLanguage::Auto);
        assert_eq!(parsed.stt.fallback_model, SttModel::SmallQ5_1);
        assert!(parsed.history.enabled);
    }

    #[test]
    fn old_config_without_new_sections_parses() {
        let text = r#"
[hotkey]
mode = "toggle"

[ui]
locale = "en"
"#;
        let settings: Settings = toml::from_str(text).unwrap();
        assert_eq!(settings.stt.model, SttModel::LargeV3TurboQ5_0);
        assert_eq!(settings.hotkey.vad_autostop_ms, 1500);
        assert!(settings.history.enabled);
    }

    #[test]
    fn vad_autostop_ms_clamped_on_load() {
        let _env_guard = ENV_LOCK.lock().unwrap();
        let dir = tempdir().unwrap();
        let config_dir = dir.path().join("lira");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(
            config_dir.join("config.toml"),
            r#"
[hotkey]
vad_autostop_ms = 9000
"#,
        )
        .unwrap();

        let prev = env::var("XDG_CONFIG_HOME").ok();
        env::set_var("XDG_CONFIG_HOME", dir.path());

        let settings = Settings::load().unwrap();
        assert_eq!(settings.hotkey.vad_autostop_ms, 5000);

        if let Some(p) = prev {
            env::set_var("XDG_CONFIG_HOME", p);
        } else {
            env::remove_var("XDG_CONFIG_HOME");
        }
    }

    #[test]
    fn effective_language_from_stt_and_ui() {
        let mut s = Settings::default();
        s.stt.language = SttLanguage::Pt;
        assert_eq!(s.effective_language(), "pt");

        s.stt.language = SttLanguage::Auto;
        s.ui.locale = UiLocale::En;
        assert_eq!(s.effective_language(), "en");

        s.ui.locale = UiLocale::PtBr;
        assert_eq!(s.effective_language(), "pt");
    }

    #[test]
    fn initial_prompt_joins_and_caps() {
        let mut s = Settings::default();
        assert!(s.initial_prompt().is_none());

        s.stt.vocabulary = vec!["lira".into(), "whisper".into()];
        assert_eq!(s.initial_prompt().as_deref(), Some("lira, whisper"));

        s.stt.vocabulary = vec!["x".repeat(120), "y".repeat(120)];
        let prompt = s.initial_prompt().unwrap();
        assert!(prompt.len() <= INITIAL_PROMPT_MAX_CHARS);
    }
}
