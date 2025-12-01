use crate::midi::error::{MidiError, MidiResult};
use crate::midi::types::{MidiDevice, MidiDeviceType, MidiMessageType};
use midir::{MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;

/// Callback type for MIDI input messages
pub type MidiInputCallback = Box<dyn Fn(MidiMessageType) + Send + 'static>;

/// Manages MIDI device connections
pub struct MidiConnectionManager {
    input_connections: Arc<RwLock<HashMap<String, MidiInputConnection<()>>>>,
    output_connections: Arc<RwLock<HashMap<String, MidiOutputConnection>>>,
    message_sender: Option<mpsc::UnboundedSender<(String, MidiMessageType)>>,
}

impl MidiConnectionManager {
    /// Create a new MidiConnectionManager
    pub fn new() -> Self {
        Self {
            input_connections: Arc::new(RwLock::new(HashMap::new())),
            output_connections: Arc::new(RwLock::new(HashMap::new())),
            message_sender: None,
        }
    }

    /// Set up a message channel for receiving MIDI input
    pub fn setup_message_channel(&mut self) -> mpsc::UnboundedReceiver<(String, MidiMessageType)> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.message_sender = Some(tx);
        rx
    }

    /// Connect to a MIDI input device
    pub fn connect_input(&self, device: &MidiDevice) -> MidiResult<()> {
        if device.device_type != MidiDeviceType::Input {
            return Err(MidiError::ConnectionError {
                device: device.name.clone(),
                reason: "Device is not an input device".to_string(),
            });
        }

        // Check if already connected
        {
            let connections = self.input_connections.read()
                .map_err(|e| MidiError::Other(format!("Failed to acquire lock: {}", e)))?;
            
            if connections.contains_key(&device.id) {
                return Ok(()); // Already connected
            }
        }

        log::info!("Connecting to MIDI input '{}' (cached port: {})", device.name, device.port_number);
        
        let mut midi_in = MidiInput::new("FaderBridge").map_err(|e| {
            log::error!("Failed to create MidiInput: {}", e);
            MidiError::InitializationError(format!("Failed to create MIDI input: {}", e))
        })?;
        
        // Don't ignore any messages - we want to receive everything
        midi_in.ignore(midir::Ignore::None);
        
        let ports = midi_in.ports();
        log::info!("Found {} MIDI input ports at connection time", ports.len());
        
        // On macOS, port_name() often fails when multiple MidiInput instances exist.
        // Use the cached port index directly instead of searching by name.
        if device.port_number >= ports.len() {
            return Err(MidiError::DeviceNotFound(format!(
                "Port {} not available (only {} ports found). Device '{}' may have been disconnected.",
                device.port_number, ports.len(), device.name
            )));
        }
        
        let port = &ports[device.port_number];
        log::info!("Using cached port index {} for '{}'", device.port_number, device.name);
        let device_id = device.id.clone();
        let sender = self.message_sender.clone();
        
        log::info!("Connecting to MIDI input '{}' (id: {}), port index: {}, sender available: {}", 
            device.name, device_id, device.port_number, sender.is_some());

        // Connect with callback - use a simple client name
        let connection = midi_in.connect(
            port,
            "FaderBridge",
            move |_timestamp, message, _| {
                // Parse and send MIDI message
                if let Some(msg_type) = MidiMessageType::from_bytes(message) {
                    if let Some(ref tx) = sender {
                        let _ = tx.send((device_id.clone(), msg_type));
                    }
                }
            },
            (),
        ).map_err(|e| {
            log::error!("Failed to connect to MIDI port {}: {}", device.port_number, e);
            MidiError::ConnectionError {
                device: device.name.clone(),
                reason: e.to_string(),
            }
        })?;

        // Store connection
        let mut connections = self.input_connections.write()
            .map_err(|e| MidiError::Other(format!("Failed to acquire lock: {}", e)))?;
        
        connections.insert(device.id.clone(), connection);

        Ok(())
    }

    /// Connect to a MIDI output device
    pub fn connect_output(&self, device: &MidiDevice) -> MidiResult<()> {
        if device.device_type != MidiDeviceType::Output {
            return Err(MidiError::ConnectionError {
                device: device.name.clone(),
                reason: "Device is not an output device".to_string(),
            });
        }

        // Check if already connected
        {
            let connections = self.output_connections.read()
                .map_err(|e| MidiError::Other(format!("Failed to acquire lock: {}", e)))?;
            
            if connections.contains_key(&device.id) {
                return Ok(()); // Already connected
            }
        }

        let midi_out = MidiOutput::new("FaderBridge-Output")?;
        let ports = midi_out.ports();
        
        if device.port_number >= ports.len() {
            return Err(MidiError::DeviceNotFound(format!(
                "Port {} not found for device '{}'",
                device.port_number, device.name
            )));
        }

        let port = &ports[device.port_number];

        // Connect
        let connection = midi_out.connect(port, &device.name)
            .map_err(|e| MidiError::ConnectionError {
                device: device.name.clone(),
                reason: e.to_string(),
            })?;

        // Store connection
        let mut connections = self.output_connections.write()
            .map_err(|e| MidiError::Other(format!("Failed to acquire lock: {}", e)))?;
        
        connections.insert(device.id.clone(), connection);

        Ok(())
    }

    /// Disconnect from a MIDI input device
    pub fn disconnect_input(&self, device_id: &str) -> MidiResult<()> {
        let mut connections = self.input_connections.write()
            .map_err(|e| MidiError::Other(format!("Failed to acquire lock: {}", e)))?;
        
        if connections.remove(device_id).is_some() {
            Ok(())
        } else {
            Err(MidiError::DeviceNotFound(device_id.to_string()))
        }
    }

    /// Disconnect from a MIDI output device
    pub fn disconnect_output(&self, device_id: &str) -> MidiResult<()> {
        let mut connections = self.output_connections.write()
            .map_err(|e| MidiError::Other(format!("Failed to acquire lock: {}", e)))?;
        
        if let Some(connection) = connections.remove(device_id) {
            connection.close();
            Ok(())
        } else {
            Err(MidiError::DeviceNotFound(device_id.to_string()))
        }
    }

    /// Send a MIDI message to an output device
    pub fn send_message(&self, device_id: &str, message: MidiMessageType) -> MidiResult<()> {
        let mut connections = self.output_connections.write()
            .map_err(|e| MidiError::Other(format!("Failed to acquire lock: {}", e)))?;
        
        if let Some(connection) = connections.get_mut(device_id) {
            let bytes = message.to_bytes();
            connection.send(&bytes)?;
            Ok(())
        } else {
            Err(MidiError::DeviceNotFound(device_id.to_string()))
        }
    }

    /// Check if a device is connected
    pub fn is_connected(&self, device_id: &str, device_type: MidiDeviceType) -> MidiResult<bool> {
        match device_type {
            MidiDeviceType::Input => {
                let connections = self.input_connections.read()
                    .map_err(|e| MidiError::Other(format!("Failed to acquire lock: {}", e)))?;
                Ok(connections.contains_key(device_id))
            }
            MidiDeviceType::Output => {
                let connections = self.output_connections.read()
                    .map_err(|e| MidiError::Other(format!("Failed to acquire lock: {}", e)))?;
                Ok(connections.contains_key(device_id))
            }
        }
    }

    /// Get the number of connected input devices
    pub fn input_count(&self) -> MidiResult<usize> {
        let connections = self.input_connections.read()
            .map_err(|e| MidiError::Other(format!("Failed to acquire lock: {}", e)))?;
        Ok(connections.len())
    }

    /// Get the number of connected output devices
    pub fn output_count(&self) -> MidiResult<usize> {
        let connections = self.output_connections.read()
            .map_err(|e| MidiError::Other(format!("Failed to acquire lock: {}", e)))?;
        Ok(connections.len())
    }

    /// Disconnect all devices
    pub fn disconnect_all(&self) -> MidiResult<()> {
        // Disconnect all inputs
        {
            let mut connections = self.input_connections.write()
                .map_err(|e| MidiError::Other(format!("Failed to acquire lock: {}", e)))?;
            connections.clear();
        }

        // Disconnect all outputs
        {
            let mut connections = self.output_connections.write()
                .map_err(|e| MidiError::Other(format!("Failed to acquire lock: {}", e)))?;
            
            for (_, connection) in connections.drain() {
                connection.close();
            }
        }

        Ok(())
    }

    /// Get IDs of all connected output devices
    pub fn get_connected_output_ids(&self) -> MidiResult<Vec<String>> {
        let connections = self.output_connections.read()
            .map_err(|e| MidiError::Other(format!("Failed to acquire lock: {}", e)))?;
        Ok(connections.keys().cloned().collect())
    }

    /// Send a MIDI message to all connected output devices
    ///
    /// Returns the number of devices the message was sent to successfully.
    /// Errors are logged but don't stop sending to other devices.
    pub fn broadcast_message(&self, message: MidiMessageType) -> MidiResult<usize> {
        let mut connections = self.output_connections.write()
            .map_err(|e| MidiError::Other(format!("Failed to acquire lock: {}", e)))?;
        
        let bytes = message.to_bytes();
        let mut success_count = 0;

        for (device_id, connection) in connections.iter_mut() {
            match connection.send(&bytes) {
                Ok(()) => {
                    success_count += 1;
                }
                Err(e) => {
                    log::warn!("Failed to send MIDI to {}: {}", device_id, e);
                }
            }
        }

        Ok(success_count)
    }
}

impl Default for MidiConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::midi::types::MidiConnectionStatus;

    #[test]
    fn test_connection_manager_creation() {
        let manager = MidiConnectionManager::new();
        assert_eq!(manager.input_count().unwrap(), 0);
        assert_eq!(manager.output_count().unwrap(), 0);
    }

    #[test]
    fn test_message_channel_setup() {
        let mut manager = MidiConnectionManager::new();
        let _rx = manager.setup_message_channel();
        assert!(manager.message_sender.is_some());
    }

    #[test]
    fn test_connect_wrong_device_type() {
        let manager = MidiConnectionManager::new();
        let device = MidiDevice {
            id: "test:0".to_string(),
            name: "Test Output".to_string(),
            manufacturer: None,
            device_type: MidiDeviceType::Output,
            port_number: 0,
            status: MidiConnectionStatus::Available,
        };

        // Try to connect output device as input
        let result = manager.connect_input(&device);
        assert!(result.is_err());
    }

    #[test]
    fn test_disconnect_nonexistent_device() {
        let manager = MidiConnectionManager::new();
        let result = manager.disconnect_input("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_send_to_nonexistent_device() {
        let manager = MidiConnectionManager::new();
        let msg = MidiMessageType::ControlChange {
            channel: 0,
            controller: 7,
            value: 100,
        };
        let result = manager.send_message("nonexistent", msg);
        assert!(result.is_err());
    }
}
