// Prevents additional console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod midi;
mod sync;
mod translation;
mod ucnet;

use commands::{MidiState, UcNetState};
use log::info;

fn main() {
    // Initialize logger
    env_logger::init();
    
    info!("Starting FaderBridge v{}", env!("CARGO_PKG_VERSION"));

    // Initialize state
    let ucnet_state = UcNetState::new();
    let midi_state = MidiState::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(ucnet_state)
        .manage(midi_state)
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::ucnet::discover_devices,
            commands::ucnet::connect_device,
            commands::ucnet::disconnect_device,
            commands::ucnet::get_connected_devices,
            commands::ucnet::get_device_state,
            commands::midi::discover_midi_devices,
            commands::midi::get_midi_devices,
            commands::midi::connect_midi_device,
            commands::midi::disconnect_midi_device,
            commands::midi::check_midi_device_changes,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
