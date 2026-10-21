pub mod history;
pub mod i18n;
pub mod ipc;
pub mod settings;
pub mod state;

pub use i18n::Locale;
pub use settings::{HoldBackend, HotkeyMode, ScreenContext, Settings, UiLocale};
pub use state::AppState;
