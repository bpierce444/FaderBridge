//! Tauri command handlers

pub mod ucnet;

// Re-export UCNet state
pub use ucnet::UcNetState;

/// Temporary greeting command for initial testing
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to FaderBridge.", name)
}
