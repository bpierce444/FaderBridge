//! Tauri command handlers

pub mod midi;
pub mod ucnet;

// Re-export state types
pub use midi::MidiState;
pub use ucnet::UcNetState;

/// Temporary greeting command for initial testing
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to FaderBridge.", name)
}
