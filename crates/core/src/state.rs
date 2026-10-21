use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppState {
    Idle,
    Listening,
    Transcribing,
}

impl AppState {
    pub fn toggle(self) -> Self {
        match self {
            Self::Idle => Self::Listening,
            Self::Listening => Self::Idle,
            Self::Transcribing => Self::Transcribing,
        }
    }
}
