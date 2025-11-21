use thiserror::Error;

/// Errors that can occur during MIDI operations
#[derive(Error, Debug)]
pub enum MidiError {
    /// Failed to initialize MIDI subsystem
    #[error("Failed to initialize MIDI: {0}")]
    InitializationError(String),

    /// Failed to enumerate MIDI devices
    #[error("Failed to enumerate MIDI devices: {0}")]
    EnumerationError(String),

    /// Failed to connect to a MIDI device
    #[error("Failed to connect to MIDI device '{device}': {reason}")]
    ConnectionError {
        device: String,
        reason: String,
    },

    /// Failed to disconnect from a MIDI device
    #[error("Failed to disconnect from MIDI device '{device}': {reason}")]
    DisconnectionError {
        device: String,
        reason: String,
    },

    /// Failed to send a MIDI message
    #[error("Failed to send MIDI message: {0}")]
    SendError(String),

    /// Failed to receive a MIDI message
    #[error("Failed to receive MIDI message: {0}")]
    ReceiveError(String),

    /// Invalid MIDI message format
    #[error("Invalid MIDI message: {0}")]
    InvalidMessage(String),

    /// Device not found
    #[error("MIDI device not found: {0}")]
    DeviceNotFound(String),

    /// Port is already in use
    #[error("MIDI port {0} is already in use")]
    PortInUse(usize),

    /// Generic MIDI error
    #[error("MIDI error: {0}")]
    Other(String),
}

impl From<midir::InitError> for MidiError {
    fn from(err: midir::InitError) -> Self {
        MidiError::InitializationError(err.to_string())
    }
}

impl From<midir::ConnectError<midir::MidiInput>> for MidiError {
    fn from(err: midir::ConnectError<midir::MidiInput>) -> Self {
        MidiError::ConnectionError {
            device: "unknown".to_string(),
            reason: err.to_string(),
        }
    }
}

impl From<midir::ConnectError<midir::MidiOutput>> for MidiError {
    fn from(err: midir::ConnectError<midir::MidiOutput>) -> Self {
        MidiError::ConnectionError {
            device: "unknown".to_string(),
            reason: err.to_string(),
        }
    }
}

impl From<midir::SendError> for MidiError {
    fn from(err: midir::SendError) -> Self {
        MidiError::SendError(err.to_string())
    }
}

/// Result type for MIDI operations
pub type MidiResult<T> = Result<T, MidiError>;
