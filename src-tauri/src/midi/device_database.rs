//! Device categorization database
//! 
//! This module provides a database of known PreSonus devices and their capabilities.
//! It's used as a fallback when heuristic-based categorization is ambiguous.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeviceCategory {
    /// MIDI controller or control surface (goes in Controllers column)
    Controller,
    /// Audio interface or mixer (goes in Mixers & Interfaces column)
    AudioInterface,
    /// Device has both capabilities (user should choose)
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// Device name or pattern
    pub name_pattern: String,
    /// Category
    pub category: DeviceCategory,
    /// Optional USB vendor ID
    pub vendor_id: Option<u16>,
    /// Optional USB product ID
    pub product_id: Option<u16>,
    /// Description
    pub description: String,
}

/// Database of known PreSonus devices
pub struct DeviceDatabase {
    devices: HashMap<String, DeviceInfo>,
}

impl DeviceDatabase {
    pub fn new() -> Self {
        let mut devices = HashMap::new();
        
        // FaderPort series (MIDI controllers)
        devices.insert("PreSonus FP2".to_string(), DeviceInfo {
            name_pattern: "PreSonus FP2".to_string(),
            category: DeviceCategory::Controller,
            vendor_id: Some(0x194f),
            product_id: Some(0x1800),
            description: "FaderPort v2 - MIDI control surface".to_string(),
        });
        
        devices.insert("PreSonus FP8".to_string(), DeviceInfo {
            name_pattern: "PreSonus FP8".to_string(),
            category: DeviceCategory::Controller,
            vendor_id: Some(0x194f),
            product_id: Some(0x1810),
            description: "FaderPort 8 - MIDI control surface".to_string(),
        });
        
        devices.insert("PreSonus FP16".to_string(), DeviceInfo {
            name_pattern: "PreSonus FP16".to_string(),
            category: DeviceCategory::Controller,
            vendor_id: Some(0x194f),
            product_id: Some(0x1820),
            description: "FaderPort 16 - MIDI control surface".to_string(),
        });
        
        // Quantum series (audio interfaces)
        devices.insert("Quantum HD 2".to_string(), DeviceInfo {
            name_pattern: "Quantum HD 2".to_string(),
            category: DeviceCategory::AudioInterface,
            vendor_id: Some(0x194f),
            product_id: Some(0x8187),
            description: "Quantum HD 2 - Thunderbolt audio interface".to_string(),
        });
        
        devices.insert("Quantum HD 4".to_string(), DeviceInfo {
            name_pattern: "Quantum HD 4".to_string(),
            category: DeviceCategory::AudioInterface,
            vendor_id: Some(0x194f),
            product_id: Some(0x8186),
            description: "Quantum HD 4 - Thunderbolt audio interface".to_string(),
        });
        
        devices.insert("Quantum HD 8".to_string(), DeviceInfo {
            name_pattern: "Quantum HD 8".to_string(),
            category: DeviceCategory::AudioInterface,
            vendor_id: Some(0x194f),
            product_id: Some(0x8188),
            description: "Quantum HD 8 - Thunderbolt audio interface".to_string(),
        });
        
        // StudioLive Series III (mixers with both audio and MIDI)
        // Note: UCNET MIDI ports are controllers, USB audio is interface
        devices.insert("UCNET MIDI StudioLive".to_string(), DeviceInfo {
            name_pattern: "UCNET MIDI StudioLive".to_string(),
            category: DeviceCategory::Controller,
            vendor_id: Some(0x194f),
            product_id: None, // Network device
            description: "StudioLive Series III - Network MIDI control".to_string(),
        });
        
        // Fender Tone Master Pro (audio interface, not a controller)
        devices.insert("Tone Master Pro".to_string(), DeviceInfo {
            name_pattern: "Tone Master Pro".to_string(),
            category: DeviceCategory::AudioInterface,
            vendor_id: None, // Fender device
            product_id: None,
            description: "Fender Tone Master Pro - USB audio interface (not a MIDI controller)".to_string(),
        });
        
        Self { devices }
    }
    
    /// Look up a device by name
    pub fn lookup_by_name(&self, name: &str) -> Option<&DeviceInfo> {
        // Exact match first
        if let Some(info) = self.devices.get(name) {
            return Some(info);
        }
        
        // Pattern match (contains)
        for (pattern, info) in &self.devices {
            if name.contains(pattern) {
                return Some(info);
            }
        }
        
        None
    }
    
    /// Look up a device by USB IDs
    pub fn lookup_by_usb_id(&self, vendor_id: u16, product_id: u16) -> Option<&DeviceInfo> {
        self.devices.values().find(|info| {
            info.vendor_id == Some(vendor_id) && info.product_id == Some(product_id)
        })
    }
    
    /// Categorize a device using heuristics and database lookup
    pub fn categorize_device(&self, name: &str, vendor_id: Option<u16>, product_id: Option<u16>) -> DeviceCategory {
        // 1. Try database lookup by USB IDs
        if let (Some(vid), Some(pid)) = (vendor_id, product_id) {
            if let Some(info) = self.lookup_by_usb_id(vid, pid) {
                return info.category.clone();
            }
        }
        
        // 2. Try database lookup by name
        if let Some(info) = self.lookup_by_name(name) {
            return info.category.clone();
        }
        
        // 3. Heuristic-based categorization
        self.categorize_by_heuristics(name)
    }
    
    /// Categorize using heuristics (pattern matching)
    fn categorize_by_heuristics(&self, name: &str) -> DeviceCategory {
        let name_lower = name.to_lowercase();
        
        // UCNET MIDI devices are always controllers
        if name.contains("UCNET MIDI") || name.contains("UCNet MIDI") {
            return DeviceCategory::Controller;
        }
        
        // Audio interfaces and devices without physical controls
        // These have MIDI ports but are NOT control surfaces
        let audio_interface_patterns = [
            "quantum", "studiolive", "audiobox", "studio 68", "studio 192",
            "scarlett", "clarett", "saffire", "liquid", // Focusrite
            "motu", "ultralite", "traveler", "828", // MOTU
            "apollo", "twin", "duo", "quad", // Universal Audio
            "tone master", "axe-fx", "helix", "kemper", "quad cortex", // Guitar processors
            "audio interface", "sound card",
        ];
        
        for pattern in &audio_interface_patterns {
            if name_lower.contains(pattern) {
                return DeviceCategory::AudioInterface;
            }
        }
        
        // Known control surface patterns (devices WITH faders/buttons)
        let controller_patterns = [
            "faderport", "x-touch", "bcf", "bcr", // Motorized faders
            "launchpad", "push", "maschine", "apc", // Pad controllers
            "keylab", "sl mk", "impulse", // Keyboard controllers with controls
            "control", "surface", "controller",
        ];
        
        for pattern in &controller_patterns {
            if name_lower.contains(pattern) {
                return DeviceCategory::Controller;
            }
        }
        
        // Default: if it's PreSonus and has "Control" or "MIDI" in name, it's a controller
        if name.contains("PreSonus") && (name.contains("Control") || name.contains("MIDI")) {
            return DeviceCategory::Controller;
        }
        
        // Conservative default: treat unknown devices as audio interfaces
        // This prevents non-control-surface devices from cluttering the Controllers list
        DeviceCategory::AudioInterface
    }
}

impl Default for DeviceDatabase {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_faderport_categorization() {
        let db = DeviceDatabase::new();
        assert_eq!(
            db.categorize_device("PreSonus FP2", Some(0x194f), Some(0x1800)),
            DeviceCategory::Controller
        );
    }
    
    #[test]
    fn test_quantum_categorization() {
        let db = DeviceDatabase::new();
        assert_eq!(
            db.categorize_device("Quantum HD 2 Control", Some(0x194f), Some(0x8187)),
            DeviceCategory::AudioInterface
        );
    }
    
    #[test]
    fn test_ucnet_midi_categorization() {
        let db = DeviceDatabase::new();
        assert_eq!(
            db.categorize_device("UCNET MIDI StudioLive 32SC Main", None, None),
            DeviceCategory::Controller
        );
    }
    
    #[test]
    fn test_heuristic_fallback() {
        let db = DeviceDatabase::new();
        // Unknown device with "UCNET MIDI" should be categorized as controller
        assert_eq!(
            db.categorize_device("UCNET MIDI Unknown Device", None, None),
            DeviceCategory::Controller
        );
    }
    
    #[test]
    fn test_tone_master_pro_categorization() {
        let db = DeviceDatabase::new();
        // Tone Master Pro should be categorized as audio interface, not controller
        assert_eq!(
            db.categorize_device("Tone Master Pro", None, None),
            DeviceCategory::AudioInterface
        );
    }
    
    #[test]
    fn test_x_touch_categorization() {
        let db = DeviceDatabase::new();
        // X-Touch is a known control surface
        assert_eq!(
            db.categorize_device("Behringer X-Touch", None, None),
            DeviceCategory::Controller
        );
    }
    
    #[test]
    fn test_conservative_default() {
        let db = DeviceDatabase::new();
        // Unknown devices default to AudioInterface to avoid clutter
        assert_eq!(
            db.categorize_device("Unknown MIDI Device", None, None),
            DeviceCategory::AudioInterface
        );
    }
}
