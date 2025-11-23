// MIDI device enumeration and communication
// This module handles MIDI device discovery, connection management, and message handling

pub mod connection;
pub mod device_database;
pub mod enumeration;
pub mod error;
pub mod learn;
pub mod mcu_protocol;
pub mod types;

// Re-export commonly used types
pub use connection::MidiConnectionManager;
pub use device_database::{DeviceCategory, DeviceDatabase, DeviceInfo};
pub use enumeration::{DeviceEnumerator, MidiDeviceManager, MidirEnumerator};
pub use error::{MidiError, MidiResult};
pub use learn::{LearnResult, LearnState, MidiLearn};
pub use mcu_protocol::{McuMessage, McuProtocol, McuToUcNetMapping};
pub use types::{MidiConnectionStatus, MidiDevice, MidiDeviceType, MidiMessageType};
