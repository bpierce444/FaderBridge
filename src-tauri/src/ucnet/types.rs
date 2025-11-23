//! UCNet protocol types and data structures

use serde::{Deserialize, Serialize};
use std::net::IpAddr;

/// Represents a discovered UCNet device
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UcNetDevice {
    /// Unique identifier for the device
    pub id: String,
    /// Device model name (e.g., "StudioLive 32SX")
    pub model: String,
    /// Firmware version
    pub firmware_version: String,
    /// Connection type
    pub connection_type: ConnectionType,
    /// Current connection state
    pub state: ConnectionState,
    /// Device-specific identifier (IP address or USB path)
    pub identifier: String,
}

/// Type of connection to the UCNet device
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConnectionType {
    /// Network connection (UDP/TCP)
    Network,
    /// USB connection
    Usb,
}

/// Current state of the device connection
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConnectionState {
    /// Device discovered but not connected
    Discovered,
    /// Attempting to connect
    Connecting,
    /// Successfully connected
    Connected,
    /// Connection lost
    Disconnected,
    /// Connection failed
    Failed,
}

/// Network device information from UDP discovery
#[derive(Debug, Clone)]
pub struct NetworkDeviceInfo {
    /// IP address of the device
    pub ip_addr: IpAddr,
    /// UDP port for communication
    pub port: u16,
    /// Device model name
    pub model: String,
    /// Firmware version
    pub firmware_version: String,
    /// Device serial number or unique ID
    pub device_id: String,
}

/// USB device information
#[derive(Debug, Clone)]
pub struct UsbDeviceInfo {
    /// USB vendor ID (should be 0x194f for PreSonus)
    pub vendor_id: u16,
    /// USB product ID
    pub product_id: u16,
    /// Device model name
    pub model: String,
    /// Firmware version
    pub firmware_version: String,
    /// USB device path/identifier
    pub device_path: String,
}

/// UCNet protocol constants
pub mod constants {
    /// PreSonus vendor ID for USB devices
    pub const PRESONUS_VENDOR_ID: u16 = 0x194f;
    
    /// Fender vendor ID (Fender owns PreSonus, some devices use this ID)
    pub const FENDER_VENDOR_ID: u16 = 0x1ED8;
    
    /// UDP port for UCNet discovery broadcasts
    pub const UCNET_DISCOVERY_PORT: u16 = 47809;
    
    /// Keep-alive interval in seconds
    pub const KEEPALIVE_INTERVAL_SECS: u64 = 5;
    
    /// Discovery timeout in seconds
    pub const DISCOVERY_TIMEOUT_SECS: u64 = 2;
    
    /// Maximum time without keep-alive before considering connection lost
    pub const CONNECTION_TIMEOUT_SECS: u64 = 15;
    
    /// Known UCNet-capable device product IDs (USB)
    /// These are the ONLY PreSonus/Fender devices that support UCNet protocol
    pub const UCNET_USB_PRODUCT_IDS: &[u16] = &[
        0x8186, // Quantum HD8 (PreSonus VID)
        0x8187, // Quantum HD4 (PreSonus VID)
        0x8188, // Quantum 26x32 (PreSonus VID)
        0x8189, // Quantum 4848 (PreSonus VID)
        0x020E, // Quantum HD 2 (Fender VID)
        // Note: Series III mixers (32SX, 24R, etc.) typically use network UCNet, not USB
        // FaderPort devices (0x1800-0x180F range) are MIDI controllers, NOT UCNet devices
    ];
    
    /// Check if a product ID is a UCNet-capable device
    pub fn is_ucnet_device(product_id: u16) -> bool {
        UCNET_USB_PRODUCT_IDS.contains(&product_id)
    }
}

impl UcNetDevice {
    /// Creates a new UCNet device from network information
    pub fn from_network(info: NetworkDeviceInfo) -> Self {
        Self {
            id: format!("net-{}", info.device_id),
            model: info.model,
            firmware_version: info.firmware_version,
            connection_type: ConnectionType::Network,
            state: ConnectionState::Discovered,
            identifier: info.ip_addr.to_string(),
        }
    }

    /// Creates a new UCNet device from USB information
    pub fn from_usb(info: UsbDeviceInfo) -> Self {
        Self {
            id: format!("usb-{}", info.device_path),
            model: info.model,
            firmware_version: info.firmware_version,
            connection_type: ConnectionType::Usb,
            state: ConnectionState::Discovered,
            identifier: info.device_path,
        }
    }
}
