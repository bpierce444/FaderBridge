//! UCNet error types

use thiserror::Error;

/// Errors that can occur during UCNet operations
#[derive(Error, Debug)]
pub enum UcNetError {
    /// Network-related errors
    #[error("Network error: {0}")]
    Network(#[from] std::io::Error),

    /// USB-related errors
    #[error("USB error: {0}")]
    Usb(#[from] rusb::Error),

    /// Device not found
    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    /// Connection timeout
    #[error("Connection timeout")]
    Timeout,

    /// Invalid device response
    #[error("Invalid device response: {0}")]
    InvalidResponse(String),

    /// Device already connected
    #[error("Device already connected: {0}")]
    AlreadyConnected(String),

    /// Connection lost
    #[error("Connection lost: {0}")]
    ConnectionLost(String),

    /// Protocol error
    #[error("Protocol error: {0}")]
    Protocol(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Result type for UCNet operations
pub type Result<T> = std::result::Result<T, UcNetError>;
