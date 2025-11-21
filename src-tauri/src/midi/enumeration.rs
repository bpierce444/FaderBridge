use crate::midi::error::{MidiError, MidiResult};
use crate::midi::types::{MidiDevice, MidiDeviceType, MidiConnectionStatus};
use midir::{MidiInput, MidiOutput};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Trait for MIDI device enumeration (enables mocking in tests)
pub trait DeviceEnumerator: Send + Sync {
    /// Discover all available MIDI input devices
    fn discover_inputs(&self) -> MidiResult<Vec<MidiDevice>>;
    
    /// Discover all available MIDI output devices
    fn discover_outputs(&self) -> MidiResult<Vec<MidiDevice>>;
    
    /// Discover all MIDI devices (inputs and outputs)
    fn discover_all(&self) -> MidiResult<Vec<MidiDevice>> {
        let mut devices = Vec::new();
        devices.extend(self.discover_inputs()?);
        devices.extend(self.discover_outputs()?);
        Ok(devices)
    }
}

/// Real MIDI device enumerator using midir
pub struct MidirEnumerator;

impl MidirEnumerator {
    /// Create a new MidirEnumerator
    pub fn new() -> Self {
        Self
    }

    /// Extract manufacturer name from device name if possible
    /// Many MIDI devices use format "Manufacturer DeviceName"
    fn extract_manufacturer(name: &str) -> Option<String> {
        // Common manufacturer prefixes
        let manufacturers = [
            "PreSonus", "Behringer", "Novation", "Akai", "M-Audio",
            "Roland", "Yamaha", "Korg", "Arturia", "Native Instruments",
            "Focusrite", "Mackie", "Allen & Heath", "Moog", "Sequential",
        ];

        for manufacturer in &manufacturers {
            if name.starts_with(manufacturer) {
                return Some(manufacturer.to_string());
            }
        }

        // Try to extract first word as manufacturer
        name.split_whitespace().next().map(|s| s.to_string())
    }

    /// Generate a unique ID for a device
    fn generate_device_id(device_type: MidiDeviceType, port: usize, name: &str) -> String {
        format!("{:?}:{}:{}", device_type, port, name)
    }
}

impl Default for MidirEnumerator {
    fn default() -> Self {
        Self::new()
    }
}

impl DeviceEnumerator for MidirEnumerator {
    fn discover_inputs(&self) -> MidiResult<Vec<MidiDevice>> {
        let midi_in = MidiInput::new("FaderBridge-Enum")?;
        let ports = midi_in.ports();
        
        let mut devices = Vec::new();
        for (idx, port) in ports.iter().enumerate() {
            let name = midi_in.port_name(port)
                .unwrap_or_else(|_| format!("MIDI Input {}", idx));
            
            let manufacturer = Self::extract_manufacturer(&name);
            let id = Self::generate_device_id(MidiDeviceType::Input, idx, &name);
            
            devices.push(MidiDevice {
                id,
                name,
                manufacturer,
                device_type: MidiDeviceType::Input,
                port_number: idx,
                status: MidiConnectionStatus::Available,
            });
        }
        
        Ok(devices)
    }

    fn discover_outputs(&self) -> MidiResult<Vec<MidiDevice>> {
        let midi_out = MidiOutput::new("FaderBridge-Enum")?;
        let ports = midi_out.ports();
        
        let mut devices = Vec::new();
        for (idx, port) in ports.iter().enumerate() {
            let name = midi_out.port_name(port)
                .unwrap_or_else(|_| format!("MIDI Output {}", idx));
            
            let manufacturer = Self::extract_manufacturer(&name);
            let id = Self::generate_device_id(MidiDeviceType::Output, idx, &name);
            
            devices.push(MidiDevice {
                id,
                name,
                manufacturer,
                device_type: MidiDeviceType::Output,
                port_number: idx,
                status: MidiConnectionStatus::Available,
            });
        }
        
        Ok(devices)
    }
}

/// Manages MIDI device discovery and hot-plug detection
pub struct MidiDeviceManager {
    enumerator: Arc<dyn DeviceEnumerator>,
    cached_devices: Arc<RwLock<HashMap<String, MidiDevice>>>,
}

impl MidiDeviceManager {
    /// Create a new MidiDeviceManager with the default enumerator
    pub fn new() -> Self {
        Self::with_enumerator(Arc::new(MidirEnumerator::new()))
    }

    /// Create a new MidiDeviceManager with a custom enumerator (for testing)
    pub fn with_enumerator(enumerator: Arc<dyn DeviceEnumerator>) -> Self {
        Self {
            enumerator,
            cached_devices: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Discover all MIDI devices and update the cache
    pub fn discover_devices(&self) -> MidiResult<Vec<MidiDevice>> {
        let devices = self.enumerator.discover_all()?;
        
        // Update cache
        let mut cache = self.cached_devices.write()
            .map_err(|e| MidiError::Other(format!("Failed to acquire cache lock: {}", e)))?;
        
        cache.clear();
        for device in &devices {
            cache.insert(device.id.clone(), device.clone());
        }
        
        Ok(devices)
    }

    /// Get a device by ID from the cache
    pub fn get_device(&self, id: &str) -> MidiResult<Option<MidiDevice>> {
        let cache = self.cached_devices.read()
            .map_err(|e| MidiError::Other(format!("Failed to acquire cache lock: {}", e)))?;
        
        Ok(cache.get(id).cloned())
    }

    /// Get all cached devices
    pub fn get_cached_devices(&self) -> MidiResult<Vec<MidiDevice>> {
        let cache = self.cached_devices.read()
            .map_err(|e| MidiError::Other(format!("Failed to acquire cache lock: {}", e)))?;
        
        Ok(cache.values().cloned().collect())
    }

    /// Update the status of a device in the cache
    pub fn update_device_status(&self, id: &str, status: MidiConnectionStatus) -> MidiResult<()> {
        let mut cache = self.cached_devices.write()
            .map_err(|e| MidiError::Other(format!("Failed to acquire cache lock: {}", e)))?;
        
        if let Some(device) = cache.get_mut(id) {
            device.status = status;
            Ok(())
        } else {
            Err(MidiError::DeviceNotFound(id.to_string()))
        }
    }

    /// Check for device changes (hot-plug detection)
    /// Returns (added_devices, removed_devices)
    pub fn check_for_changes(&self) -> MidiResult<(Vec<MidiDevice>, Vec<MidiDevice>)> {
        let current_devices = self.enumerator.discover_all()?;
        let cache = self.cached_devices.read()
            .map_err(|e| MidiError::Other(format!("Failed to acquire cache lock: {}", e)))?;
        
        let current_ids: HashMap<String, MidiDevice> = current_devices
            .iter()
            .map(|d| (d.id.clone(), d.clone()))
            .collect();
        
        let cached_ids: HashMap<String, MidiDevice> = cache
            .iter()
            .map(|(id, d)| (id.clone(), d.clone()))
            .collect();
        
        // Find added devices
        let added: Vec<MidiDevice> = current_ids
            .iter()
            .filter(|(id, _)| !cached_ids.contains_key(*id))
            .map(|(_, d)| d.clone())
            .collect();
        
        // Find removed devices
        let removed: Vec<MidiDevice> = cached_ids
            .iter()
            .filter(|(id, _)| !current_ids.contains_key(*id))
            .map(|(_, d)| d.clone())
            .collect();
        
        Ok((added, removed))
    }
}

impl Default for MidiDeviceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockEnumerator {
        inputs: Vec<MidiDevice>,
        outputs: Vec<MidiDevice>,
    }

    impl DeviceEnumerator for MockEnumerator {
        fn discover_inputs(&self) -> MidiResult<Vec<MidiDevice>> {
            Ok(self.inputs.clone())
        }

        fn discover_outputs(&self) -> MidiResult<Vec<MidiDevice>> {
            Ok(self.outputs.clone())
        }
    }

    #[test]
    fn test_discover_devices() {
        let mock = Arc::new(MockEnumerator {
            inputs: vec![MidiDevice {
                id: "input:0:Test Input".to_string(),
                name: "Test Input".to_string(),
                manufacturer: Some("Test".to_string()),
                device_type: MidiDeviceType::Input,
                port_number: 0,
                status: MidiConnectionStatus::Available,
            }],
            outputs: vec![MidiDevice {
                id: "output:0:Test Output".to_string(),
                name: "Test Output".to_string(),
                manufacturer: Some("Test".to_string()),
                device_type: MidiDeviceType::Output,
                port_number: 0,
                status: MidiConnectionStatus::Available,
            }],
        });

        let manager = MidiDeviceManager::with_enumerator(mock);
        let devices = manager.discover_devices().unwrap();
        
        assert_eq!(devices.len(), 2);
        assert_eq!(devices[0].device_type, MidiDeviceType::Input);
        assert_eq!(devices[1].device_type, MidiDeviceType::Output);
    }

    #[test]
    fn test_update_device_status() {
        let mock = Arc::new(MockEnumerator {
            inputs: vec![MidiDevice {
                id: "input:0:Test".to_string(),
                name: "Test".to_string(),
                manufacturer: None,
                device_type: MidiDeviceType::Input,
                port_number: 0,
                status: MidiConnectionStatus::Available,
            }],
            outputs: vec![],
        });

        let manager = MidiDeviceManager::with_enumerator(mock);
        manager.discover_devices().unwrap();
        
        manager.update_device_status("input:0:Test", MidiConnectionStatus::Connected).unwrap();
        
        let device = manager.get_device("input:0:Test").unwrap().unwrap();
        assert_eq!(device.status, MidiConnectionStatus::Connected);
    }

    #[test]
    fn test_extract_manufacturer() {
        assert_eq!(
            MidirEnumerator::extract_manufacturer("PreSonus FaderPort"),
            Some("PreSonus".to_string())
        );
        assert_eq!(
            MidirEnumerator::extract_manufacturer("Behringer X-Touch"),
            Some("Behringer".to_string())
        );
    }
}
