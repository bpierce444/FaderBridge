/// Temporary greeting command for initial testing
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to FaderBridge.", name)
}
