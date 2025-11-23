use crate::midi::{MidiDevice, MidiDeviceManager, MidiConnectionManager, MidiDeviceType, MidiConnectionStatus};
use std::sync::{Arc, Mutex};
use tauri::State;

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

/// Discover all available MIDI devices
#[tauri::command]
pub async fn discover_midi_devices(
    state: State<'_, MidiState>,
) -> Result<Vec<MidiDevice>, String> {
    let manager = state.device_manager.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    
    manager.discover_devices()
        .map_err(|e| e.to_string())
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
    // Extract device name from ID (format is "Type:Port:Name")
    let device_name = device_id.split(':').nth(2)
        .ok_or_else(|| format!("Invalid device ID format: {}", device_id))?;
    
    // Refresh device list to get current port numbers
    {
        let device_manager = state.device_manager.lock()
            .map_err(|e| format!("Failed to acquire lock: {}", e))?;
        
        device_manager.discover_devices()
            .map_err(|e| format!("Failed to refresh devices: {}", e))?;
    }
    
    // Find device by name and type (port number may have changed)
    let device = {
        let device_manager = state.device_manager.lock()
            .map_err(|e| format!("Failed to acquire lock: {}", e))?;
        
        let devices = device_manager.get_cached_devices()
            .map_err(|e| e.to_string())?;
        
        devices.into_iter()
            .find(|d| d.name == device_name && d.device_type == device_type)
            .ok_or_else(|| format!("Device not found: {}", device_name))?
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

    // Update device status using the fresh device ID
    let device_manager = state.device_manager.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    
    device_manager.update_device_status(&device.id, MidiConnectionStatus::Connected)
        .map_err(|e| e.to_string())?;

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
