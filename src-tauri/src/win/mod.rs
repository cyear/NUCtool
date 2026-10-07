pub mod permissions;
pub mod task;
pub mod language;
pub mod keyboard_registry;
pub use permissions::privilege_escalation;
pub use task::*;
pub use language::get_windows_language;