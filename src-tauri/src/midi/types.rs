use serde::{Deserialize, Serialize};

/// Represents a MIDI device (input or output)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MidiDevice {
    /// Unique identifier for this device
    pub id: String,
    /// Human-readable name of the device
    pub name: String,
    /// Manufacturer name (if available)
    pub manufacturer: Option<String>,
    /// Device type (input or output)
    pub device_type: MidiDeviceType,
    /// Port number in the system
    pub port_number: usize,
    /// Connection status
    pub status: MidiConnectionStatus,
}

/// Type of MIDI device
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MidiDeviceType {
    /// MIDI input device (receives MIDI messages)
    Input,
    /// MIDI output device (sends MIDI messages)
    Output,
}

/// Connection status of a MIDI device
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MidiConnectionStatus {
    /// Device is available but not connected
    Available,
    /// Device is currently connected
    Connected,
    /// Device is disconnected or unavailable
    Disconnected,
}

/// MIDI message types we care about for control surfaces
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MidiMessageType {
    /// Control Change (CC) message
    ControlChange { channel: u8, controller: u8, value: u8 },
    /// Note On message
    NoteOn { channel: u8, note: u8, velocity: u8 },
    /// Note Off message
    NoteOff { channel: u8, note: u8, velocity: u8 },
    /// Pitch Bend message
    PitchBend { channel: u8, value: u16 },
    /// Program Change message
    ProgramChange { channel: u8, program: u8 },
}

impl MidiMessageType {
    /// Parse a raw MIDI message into a MidiMessageType
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.is_empty() {
            return None;
        }

        let status = bytes[0];
        let message_type = status & 0xF0;
        let channel = status & 0x0F;

        match message_type {
            0x80 => {
                // Note Off
                if bytes.len() >= 3 {
                    Some(MidiMessageType::NoteOff {
                        channel,
                        note: bytes[1],
                        velocity: bytes[2],
                    })
                } else {
                    None
                }
            }
            0x90 => {
                // Note On
                if bytes.len() >= 3 {
                    Some(MidiMessageType::NoteOn {
                        channel,
                        note: bytes[1],
                        velocity: bytes[2],
                    })
                } else {
                    None
                }
            }
            0xB0 => {
                // Control Change
                if bytes.len() >= 3 {
                    Some(MidiMessageType::ControlChange {
                        channel,
                        controller: bytes[1],
                        value: bytes[2],
                    })
                } else {
                    None
                }
            }
            0xC0 => {
                // Program Change
                if bytes.len() >= 2 {
                    Some(MidiMessageType::ProgramChange {
                        channel,
                        program: bytes[1],
                    })
                } else {
                    None
                }
            }
            0xE0 => {
                // Pitch Bend
                if bytes.len() >= 3 {
                    let value = ((bytes[2] as u16) << 7) | (bytes[1] as u16);
                    Some(MidiMessageType::PitchBend { channel, value })
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Convert a MidiMessageType to raw MIDI bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        match *self {
            MidiMessageType::NoteOff { channel, note, velocity } => {
                vec![0x80 | channel, note, velocity]
            }
            MidiMessageType::NoteOn { channel, note, velocity } => {
                vec![0x90 | channel, note, velocity]
            }
            MidiMessageType::ControlChange { channel, controller, value } => {
                vec![0xB0 | channel, controller, value]
            }
            MidiMessageType::ProgramChange { channel, program } => {
                vec![0xC0 | channel, program]
            }
            MidiMessageType::PitchBend { channel, value } => {
                vec![0xE0 | channel, (value & 0x7F) as u8, ((value >> 7) & 0x7F) as u8]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_control_change() {
        let bytes = [0xB0, 0x07, 0x64]; // CC 7 (volume) on channel 0, value 100
        let msg = MidiMessageType::from_bytes(&bytes);
        assert_eq!(
            msg,
            Some(MidiMessageType::ControlChange {
                channel: 0,
                controller: 7,
                value: 100
            })
        );
    }

    #[test]
    fn test_parse_note_on() {
        let bytes = [0x91, 0x3C, 0x7F]; // Note On, channel 1, middle C, max velocity
        let msg = MidiMessageType::from_bytes(&bytes);
        assert_eq!(
            msg,
            Some(MidiMessageType::NoteOn {
                channel: 1,
                note: 60,
                velocity: 127
            })
        );
    }

    #[test]
    fn test_parse_pitch_bend() {
        let bytes = [0xE0, 0x00, 0x40]; // Pitch bend, channel 0, center position
        let msg = MidiMessageType::from_bytes(&bytes);
        assert_eq!(
            msg,
            Some(MidiMessageType::PitchBend {
                channel: 0,
                value: 0x2000
            })
        );
    }

    #[test]
    fn test_to_bytes_control_change() {
        let msg = MidiMessageType::ControlChange {
            channel: 0,
            controller: 7,
            value: 100,
        };
        assert_eq!(msg.to_bytes(), vec![0xB0, 0x07, 0x64]);
    }

    #[test]
    fn test_to_bytes_note_on() {
        let msg = MidiMessageType::NoteOn {
            channel: 1,
            note: 60,
            velocity: 127,
        };
        assert_eq!(msg.to_bytes(), vec![0x91, 0x3C, 0x7F]);
    }
}
