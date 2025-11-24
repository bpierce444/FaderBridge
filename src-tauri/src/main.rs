// Prevents additional console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod midi;
mod sync;
mod translation;
mod ucnet;

use commands::{AppState, MidiLearnState, MidiState, SyncState, UcNetState};
use db::Database;
use log::{error, info};
use std::sync::Arc;

fn main() {
    // Initialize logger
    env_logger::init();
    
    info!("Starting FaderBridge v{}", env!("CARGO_PKG_VERSION"));

    // Initialize database
    let db_path = match Database::default_path() {
        Ok(path) => path,
        Err(e) => {
            error!("Failed to get database path: {}", e);
            panic!("Cannot start without database");
        }
    };

    info!("Initializing database at {:?}", db_path);
    let db = match Database::init(db_path) {
        Ok(db) => db,
        Err(e) => {
            error!("Failed to initialize database: {}", e);
            panic!("Cannot start without database");
        }
    };

    if let Err(e) = db.migrate() {
        error!("Failed to run database migrations: {}", e);
        panic!("Cannot start without database migrations");
    }

    // Initialize state
    let app_state = AppState { db: Arc::new(db) };
    let ucnet_state = UcNetState::new();
    let midi_state = MidiState::new();
    let sync_state = SyncState::new();
    let learn_state = MidiLearnState::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .manage(ucnet_state)
        .manage(midi_state)
        .manage(sync_state)
        .manage(learn_state)
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            // UCNet commands
            commands::ucnet::discover_devices,
            commands::ucnet::connect_device,
            commands::ucnet::disconnect_device,
            commands::ucnet::get_connected_devices,
            commands::ucnet::get_device_state,
            // MIDI commands
            commands::midi::discover_midi_devices,
            commands::midi::get_midi_devices,
            commands::midi::connect_midi_device,
            commands::midi::disconnect_midi_device,
            commands::midi::check_midi_device_changes,
            // Sync commands
            commands::sync::init_sync_engine,
            commands::sync::add_parameter_mapping,
            commands::sync::remove_parameter_mapping,
            commands::sync::clear_parameter_mappings,
            commands::sync::get_parameter_mappings,
            commands::sync::get_latency_stats,
            commands::sync::clear_latency_stats,
            commands::sync::clear_device_state,
            commands::sync::clear_all_state,
            // MIDI Learn commands
            commands::learn::start_midi_learn,
            commands::learn::cancel_midi_learn,
            commands::learn::get_midi_learn_state,
            commands::learn::is_midi_learning,
            // Project commands
            commands::projects::create_project,
            commands::projects::get_project,
            commands::projects::get_all_projects,
            commands::projects::get_recent_projects,
            commands::projects::update_project,
            commands::projects::set_active_project,
            commands::projects::get_active_project,
            commands::projects::delete_project,
            // Device commands
            commands::projects::create_device,
            commands::projects::get_device,
            commands::projects::get_devices_by_project,
            commands::projects::update_device_config,
            commands::projects::delete_device,
            // Mapping commands
            commands::projects::create_mapping,
            commands::projects::get_mapping,
            commands::projects::get_mappings_by_project,
            commands::projects::update_mapping,
            commands::projects::delete_mapping,
            // Sync integration commands
            commands::sync_integration::start_sync_integration,
            commands::sync_integration::stop_sync_integration,
            commands::sync_integration::trigger_midi_sync,
            commands::sync_integration::get_sync_status,
            // Export/Import commands
            commands::projects::export_project,
            commands::projects::export_project_to_file,
            commands::projects::import_project,
            commands::projects::import_project_from_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
