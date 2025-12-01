use crate::midi::{MidiDevice, MidiDeviceManager, MidiConnectionManager, MidiDeviceType, MidiConnectionStatus, MidiMessageType};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter, State};

/// Flag to track if MIDI monitoring is already running
static MIDI_MONITOR_RUNNING: AtomicBool = AtomicBool::new(false);

/// Global MIDI device manager state
pub struct MidiState {
    pub device_manager: Arc<Mutex<MidiDeviceManager>>,
    pub connection_manager: Arc<Mutex<MidiConnectionManager>>,
}

impl MidiState {
    pub fn new() -> Self {
        Self {
            device_manager: Arc::new(Mutex::new(MidiDeviceManager::new())),
            connection_manager: Arc::new(Mutex::new(MidiConnectionManager::new())),
        }
    }
}

/// Start MIDI message monitoring
/// 
/// Sets up a background task that listens for MIDI messages and emits them
/// to the frontend for the Message Monitor. This runs independently of sync.
#[tauri::command]
pub async fn start_midi_monitoring(
    app_handle: AppHandle,
    state: State<'_, MidiState>,
) -> Result<(), String> {
    // Check if already running
    if MIDI_MONITOR_RUNNING.swap(true, Ordering::SeqCst) {
        log::info!("MIDI monitoring already running");
        return Ok(());
    }

    log::info!("Starting MIDI message monitoring...");

    // Set up message channel
    let mut midi_rx = {
        let mut conn_manager = state
            .connection_manager
            .lock()
            .map_err(|e| format!("Failed to acquire MIDI lock: {}", e))?;
        conn_manager.setup_message_channel()
    };

    // Spawn task to handle MIDI messages
    tokio::spawn(async move {
        log::info!("MIDI monitor task started");

        while let Some((device_id, message)) = midi_rx.recv().await {
            log::debug!("MIDI monitor received: {:?} from {}", message, device_id);

            // Format message for frontend
            let payload = match &message {
                MidiMessageType::ControlChange { channel, controller, value } => {
                    serde_json::json!({
                        "device_id": device_id,
                        "type": "control_change",
                        "channel": channel,
                        "controller": controller,
                        "value": value,
                    })
                }
                MidiMessageType::NoteOn { channel, note, velocity } => {
                    serde_json::json!({
                        "device_id": device_id,
                        "type": "note_on",
                        "channel": channel,
                        "note": note,
                        "velocity": velocity,
                    })
                }
                MidiMessageType::NoteOff { channel, note, velocity } => {
                    serde_json::json!({
                        "device_id": device_id,
                        "type": "note_off",
                        "channel": channel,
                        "note": note,
                        "velocity": velocity,
                    })
                }
                MidiMessageType::PitchBend { channel, value } => {
                    serde_json::json!({
                        "device_id": device_id,
                        "type": "pitch_bend",
                        "channel": channel,
                        "value": value,
                    })
                }
                MidiMessageType::ProgramChange { channel, program } => {
                    serde_json::json!({
                        "device_id": device_id,
                        "type": "program_change",
                        "channel": channel,
                        "program": program,
                    })
                }
            };

            if let Err(e) = app_handle.emit("midi:message-received", payload) {
                log::error!("Failed to emit MIDI message event: {}", e);
            }
        }

        log::info!("MIDI monitor task stopped");
        MIDI_MONITOR_RUNNING.store(false, Ordering::SeqCst);
    });

    Ok(())
}

/// Discover all available MIDI devices
/// 
/// On macOS, CoreMIDI has issues with multiple MidiInput/MidiOutput instances.
/// After the initial enumeration, this returns cached devices to avoid failures.
/// The cache is populated on app startup.
#[tauri::command]
pub async fn discover_midi_devices(
    state: State<'_, MidiState>,
) -> Result<Vec<MidiDevice>, String> {
    log::info!("discover_midi_devices called");
    
    let manager = state.device_manager.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    
    // Check if we already have cached devices
    let cached = manager.get_cached_devices()
        .map_err(|e| e.to_string())?;
    
    if !cached.is_empty() {
        // Return cached devices - re-enumeration often fails on macOS
        log::info!("Returning {} cached MIDI devices", cached.len());
        return Ok(cached);
    }
    
    // First enumeration - populate the cache
    let devices = manager.discover_devices()
        .map_err(|e| e.to_string())?;
    
    log::info!("Discovered {} MIDI devices: {:?}", devices.len(), 
        devices.iter().map(|d| &d.name).collect::<Vec<_>>());
    
    Ok(devices)
}

/// Get all cached MIDI devices
#[tauri::command]
pub async fn get_midi_devices(
    state: State<'_, MidiState>,
) -> Result<Vec<MidiDevice>, String> {
    let manager = state.device_manager.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    
    manager.get_cached_devices()
        .map_err(|e| e.to_string())
}

/// Connect to a MIDI device
#[tauri::command]
pub async fn connect_midi_device(
    device_id: String,
    device_type: MidiDeviceType,
    state: State<'_, MidiState>,
) -> Result<(), String> {
    log::info!("connect_midi_device called with id='{}', type={:?}", device_id, device_type);
    
    // Extract port number and device name from ID (format is "Type:Port:Name")
    let parts: Vec<&str> = device_id.splitn(3, ':').collect();
    if parts.len() < 3 {
        return Err(format!("Invalid device ID format: {}", device_id));
    }
    let port_number: usize = parts[1].parse()
        .map_err(|_| format!("Invalid port number in device ID: {}", device_id))?;
    let device_name = parts[2];
    log::info!("Extracted device name: '{}', port: {}", device_name, port_number);
    
    // Create device directly from the ID info - don't re-enumerate
    // Re-enumeration fails on macOS when another MIDI connection is open
    let device = MidiDevice {
        id: device_id.clone(),
        name: device_name.to_string(),
        manufacturer: None,
        device_type: device_type.clone(),
        port_number,
        status: MidiConnectionStatus::Available,
    };

    // Connect to device
    let connection_manager = state.connection_manager.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    
    match device_type {
        MidiDeviceType::Input => {
            connection_manager.connect_input(&device)
                .map_err(|e| e.to_string())?;
        }
        MidiDeviceType::Output => {
            connection_manager.connect_output(&device)
                .map_err(|e| e.to_string())?;
        }
    }

    // Update device status in cache (ignore errors - device might not be in cache)
    if let Ok(device_manager) = state.device_manager.lock() {
        let _ = device_manager.update_device_status(&device.id, MidiConnectionStatus::Connected);
    }
    
    log::info!("Successfully connected to MIDI device: {}", device_name);
    Ok(())
}

/// Disconnect from a MIDI device
#[tauri::command]
pub async fn disconnect_midi_device(
    device_id: String,
    device_type: MidiDeviceType,
    state: State<'_, MidiState>,
) -> Result<(), String> {
    // Disconnect from device
    let connection_manager = state.connection_manager.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    
    match device_type {
        MidiDeviceType::Input => {
            connection_manager.disconnect_input(&device_id)
                .map_err(|e| e.to_string())?;
        }
        MidiDeviceType::Output => {
            connection_manager.disconnect_output(&device_id)
                .map_err(|e| e.to_string())?;
        }
    }

    // Update device status
    let device_manager = state.device_manager.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    
    device_manager.update_device_status(&device_id, MidiConnectionStatus::Available)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Check for device changes (hot-plug detection)
#[tauri::command]
pub async fn check_midi_device_changes(
    state: State<'_, MidiState>,
) -> Result<(Vec<MidiDevice>, Vec<MidiDevice>), String> {
    let manager = state.device_manager.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    
    manager.check_for_changes()
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi_state_creation() {
        let state = MidiState::new();
        let manager = state.device_manager.lock().unwrap();
        let devices = manager.get_cached_devices().unwrap();
        assert_eq!(devices.len(), 0);
    }

    #[test]
    fn test_connection_manager_creation() {
        let state = MidiState::new();
        let conn_manager = state.connection_manager.lock().unwrap();
        assert_eq!(conn_manager.input_count().unwrap(), 0);
        assert_eq!(conn_manager.output_count().unwrap(), 0);
    }
}
