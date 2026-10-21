use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppState {
    Idle,
    Listening,
    Transcribing,
    WaitingAgent,
}

impl AppState {
    pub fn toggle(self) -> Self {
        match self {
            Self::Idle => Self::Listening,
            Self::Listening => Self::Idle,
            Self::Transcribing => Self::Transcribing,
            Self::WaitingAgent => Self::WaitingAgent,
        }
    }
}
