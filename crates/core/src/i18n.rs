use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    PtBr,
    En,
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

    pub fn not_running(self) -> &'static str {
        match self {
            Self::PtBr => "Lira não está em execução",
            Self::En => "Lira is not running",
        }
    }

    pub fn state_idle(self) -> &'static str {
        match self {
            Self::PtBr => "Ocioso",
            Self::En => "Idle",
        }
    }

    pub fn state_listening(self) -> &'static str {
        match self {
            Self::PtBr => "Ouvindo…",
            Self::En => "Listening…",
        }
    }

    pub fn status_line(self, state: crate::AppState) -> String {
        match state {
            crate::AppState::Idle => self.state_idle().to_string(),
            crate::AppState::Listening => self.state_listening().to_string(),
        }
    }

    pub fn command_ok(self, cmd: &str) -> String {
        match self {
            Self::PtBr => format!("Comando '{}' aceito", cmd),
            Self::En => format!("Command '{}' accepted", cmd),
        }
    }
}
