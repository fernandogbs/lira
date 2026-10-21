//! Approval policy for voice actions.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VoiceApprove {
    #[default]
    Off,
    Reversible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionClass {
    Read,
    Reversible,
    Destructive,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyDecision {
    Allow,
    RequireClick,
    DenyVoice,
}

/// Classify a free-text approval target / instruction heuristically.
pub fn classify_action(text: &str) -> ActionClass {
    let lower = text.to_lowercase();
    const DESTRUCTIVE: &[&str] = &[
        "rm ",
        "delete",
        "drop ",
        "migrate",
        "push --force",
        "force push",
        "deploy",
        "production",
        "prod ",
        "enviar",
        "send message",
        "git push",
        "reset --hard",
    ];
    if DESTRUCTIVE.iter().any(|k| lower.contains(k)) {
        return ActionClass::Destructive;
    }
    const REVERSIBLE: &[&str] = &[
        "test", "teste", "branch", "commit", "format", "lint", "build",
    ];
    if REVERSIBLE.iter().any(|k| lower.contains(k)) {
        return ActionClass::Reversible;
    }
    ActionClass::Read
}

pub fn decide_voice_approval(policy: VoiceApprove, class: ActionClass) -> PolicyDecision {
    match class {
        ActionClass::Destructive => PolicyDecision::DenyVoice,
        ActionClass::Read => PolicyDecision::Allow,
        ActionClass::Reversible => match policy {
            VoiceApprove::Off => PolicyDecision::RequireClick,
            VoiceApprove::Reversible => PolicyDecision::Allow,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn destructive_never_voice() {
        assert_eq!(
            decide_voice_approval(VoiceApprove::Reversible, classify_action("deploy to prod")),
            PolicyDecision::DenyVoice
        );
    }

    #[test]
    fn reversible_off_needs_click() {
        assert_eq!(
            decide_voice_approval(VoiceApprove::Off, classify_action("run the tests")),
            PolicyDecision::RequireClick
        );
    }
}
