//! Tauri command handlers

pub mod learn;
pub mod midi;
pub mod projects;
pub mod sync;
pub mod ucnet;

// Re-export state types
pub use learn::MidiLearnState;
pub use midi::MidiState;
pub use projects::AppState;
pub use sync::SyncState;
pub use ucnet::UcNetState;

/// Temporary greeting command for initial testing
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to FaderBridge.", name)
}
