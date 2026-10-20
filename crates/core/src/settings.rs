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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub hotkey_mode: HotkeyMode,
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
