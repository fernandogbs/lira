pub mod history;
pub mod i18n;
pub mod ipc;
pub mod policy;
pub mod router;
pub mod settings;
pub mod state;

pub use i18n::Locale;
pub use policy::{decide_voice_approval, ActionClass, PolicyDecision, VoiceApprove};
pub use router::{route, Intent};
pub use settings::{
    HoldBackend, HotkeyMode, ScreenContext, ScreenPreview, ScreenScope, Settings, UiLocale,
};
pub use state::AppState;
