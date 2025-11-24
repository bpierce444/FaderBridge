//! UCNet protocol implementation
//!
//! This module handles device discovery and communication with PreSonus UCNet devices,
//! supporting both network (UDP/TCP) and USB connections.
//!
//! ## Protocol Details
//! - TCP port 53000 for control communication
//! - UDP port 47809 for discovery broadcasts
//! - Protocol derived from reverse engineering by featherbear
//!   (https://featherbear.cc/presonus-studiolive-api/)

pub mod connection;
pub mod discovery;
pub mod error;
pub mod protocol;
pub mod types;

// Re-export commonly used types
pub use connection::{ConnectionEvent, ConnectionManager};
pub use discovery::{DefaultDeviceDiscovery, DeviceDiscovery};
pub use error::{Result, UcNetError};
pub use types::{ConnectionState, ConnectionType, UcNetDevice};
