//! UCNet protocol implementation
//!
//! This module handles device discovery and communication with PreSonus UCNet devices,
//! supporting both network (UDP/TCP) and USB connections.

pub mod connection;
pub mod discovery;
pub mod error;
pub mod types;

// Re-export commonly used types
pub use connection::{ConnectionEvent, ConnectionManager};
pub use discovery::{DefaultDeviceDiscovery, DeviceDiscovery};
pub use error::{Result, UcNetError};
pub use types::{ConnectionState, ConnectionType, UcNetDevice};
