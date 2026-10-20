use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HotkeySection {
    #[serde(default)]
    pub mode: HotkeyMode,
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub hotkey: HotkeySection,
    #[serde(default)]
    pub ui: UiSection,
    #[serde(default)]
    pub screen: ScreenSection,
}

impl Settings {
    pub fn hotkey_mode(&self) -> HotkeyMode {
        self.hotkey.mode
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
        let settings: Self = toml::from_str(&text)?;
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
    use tempfile::tempdir;

    #[test]
    fn parses_sectioned_config() {
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
        assert_eq!(parsed.ui.locale, UiLocale::System);
        assert_eq!(parsed.screen.context, ScreenContext::Off);
    }
}
