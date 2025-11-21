//! Mackie Control Universal (MCU) Protocol Implementation
//!
//! This module handles translation of MCU protocol messages to standard MIDI CC
//! and provides MCU-specific functionality like motorized fader feedback, LED control,
//! and display updates.
//!
//! MCU is the most widely supported protocol for DAW controllers and is used by:
//! - Behringer X-Touch
//! - PreSonus FaderPort (MCU mode)
//! - Icon Platform series
//! - Many other control surfaces

use serde::{Deserialize, Serialize};

/// MCU channel strip controls (per channel)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McuControl {
    /// Channel fader (14-bit pitch bend on channel N)
    Fader(u8),
    /// V-Pot (rotary encoder) - CC 16-23
    VPot(u8),
    /// V-Pot LED ring mode - CC 48-55
    VPotLed(u8),
    /// Record Ready button - Note 0-7
    RecordReady(u8),
    /// Solo button - Note 8-15
    Solo(u8),
    /// Mute button - Note 16-23
    Mute(u8),
    /// Select button - Note 24-31
    Select(u8),
}

/// MCU transport controls
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McuTransport {
    /// Rewind - Note 91
    Rewind,
    /// Fast Forward - Note 92
    FastForward,
    /// Stop - Note 93
    Stop,
    /// Play - Note 94
    Play,
    /// Record - Note 95
    Record,
}

/// MCU display segment (7-segment LCD)
#[derive(Debug, Clone)]
pub struct McuDisplay {
    /// Position (0-111, 2 rows of 56 characters)
    pub position: u8,
    /// Text to display
    pub text: String,
}

/// MCU fader position (14-bit)
#[derive(Debug, Clone, Copy)]
pub struct McuFaderPosition {
    /// Channel number (0-7 for main unit, 8-15 for extender)
    pub channel: u8,
    /// 14-bit position (0-16383)
    pub position: u16,
}

/// MCU V-Pot LED ring mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VPotLedMode {
    /// Single dot
    Single,
    /// Boost/Cut (center LED)
    BoostCut,
    /// Wrap (circular)
    Wrap,
    /// Spread (from center)
    Spread,
}

/// MCU message type
#[derive(Debug, Clone)]
pub enum McuMessage {
    /// Fader movement (pitch bend)
    Fader(McuFaderPosition),
    /// V-Pot rotation (relative CC)
    VPot { channel: u8, delta: i8 },
    /// Button press/release
    Button { note: u8, pressed: bool },
    /// Display update
    Display(McuDisplay),
    /// V-Pot LED update
    VPotLed { channel: u8, mode: VPotLedMode, value: u8 },
}

/// MCU protocol handler
pub struct McuProtocol {
    /// Current fader positions (14-bit)
    fader_positions: [u16; 8],
    /// Current V-Pot values
    vpot_values: [u8; 8],
    /// Button states
    button_states: [bool; 128],
}

impl McuProtocol {
    /// Creates a new MCU protocol handler
    pub fn new() -> Self {
        Self {
            fader_positions: [8192; 8], // Center position
            vpot_values: [64; 8],        // Center position
            button_states: [false; 128],
        }
    }

    /// Parse incoming MIDI message as MCU protocol
    pub fn parse_midi(&mut self, status: u8, data1: u8, data2: u8) -> Option<McuMessage> {
        let message_type = status & 0xF0;
        let channel = status & 0x0F;

        match message_type {
            // Pitch Bend (Faders)
            0xE0 => {
                let position = ((data2 as u16) << 7) | (data1 as u16);
                self.fader_positions[channel as usize] = position;
                Some(McuMessage::Fader(McuFaderPosition {
                    channel,
                    position,
                }))
            }

            // Control Change (V-Pots and LEDs)
            0xB0 => {
                if (16..=23).contains(&data1) {
                    // V-Pot rotation (relative encoding)
                    let vpot_channel = data1 - 16;
                    let delta = if data2 & 0x40 != 0 {
                        // Counter-clockwise (negative)
                        -((data2 & 0x3F) as i8)
                    } else {
                        // Clockwise (positive)
                        (data2 & 0x3F) as i8
                    };
                    Some(McuMessage::VPot {
                        channel: vpot_channel,
                        delta,
                    })
                } else if (48..=55).contains(&data1) {
                    // V-Pot LED ring update
                    let vpot_channel = data1 - 48;
                    let mode = match (data2 >> 4) & 0x03 {
                        0 => VPotLedMode::Single,
                        1 => VPotLedMode::BoostCut,
                        2 => VPotLedMode::Wrap,
                        3 => VPotLedMode::Spread,
                        _ => VPotLedMode::Single,
                    };
                    let value = data2 & 0x0F;
                    Some(McuMessage::VPotLed {
                        channel: vpot_channel,
                        mode,
                        value,
                    })
                } else {
                    None
                }
            }

            // Note On/Off (Buttons)
            0x90 | 0x80 => {
                let pressed = message_type == 0x90 && data2 > 0;
                self.button_states[data1 as usize] = pressed;
                Some(McuMessage::Button {
                    note: data1,
                    pressed,
                })
            }

            _ => None,
        }
    }

    /// Convert MCU fader position (14-bit) to normalized value (0.0-1.0)
    pub fn fader_to_normalized(position: u16) -> f32 {
        (position as f32) / 16383.0
    }

    /// Convert normalized value (0.0-1.0) to MCU fader position (14-bit)
    pub fn normalized_to_fader(value: f32) -> u16 {
        (value.clamp(0.0, 1.0) * 16383.0) as u16
    }

    /// Convert MCU V-Pot delta to normalized delta
    pub fn vpot_delta_to_normalized(delta: i8) -> f32 {
        (delta as f32) / 64.0 // Typical V-Pot sends ±1 to ±15
    }

    /// Generate MIDI messages to update fader position (for motorized faders)
    pub fn create_fader_update(&self, channel: u8, position: u16) -> Vec<u8> {
        let lsb = (position & 0x7F) as u8;
        let msb = ((position >> 7) & 0x7F) as u8;
        vec![0xE0 | channel, lsb, msb]
    }

    /// Generate MIDI messages to update V-Pot LED ring
    pub fn create_vpot_led_update(&self, channel: u8, mode: VPotLedMode, value: u8) -> Vec<u8> {
        let mode_bits = match mode {
            VPotLedMode::Single => 0,
            VPotLedMode::BoostCut => 1,
            VPotLedMode::Wrap => 2,
            VPotLedMode::Spread => 3,
        };
        let data = ((mode_bits & 0x03) << 4) | (value & 0x0F);
        vec![0xB0, 48 + channel, data]
    }

    /// Generate MIDI messages to update button LED
    pub fn create_button_led_update(&self, note: u8, on: bool) -> Vec<u8> {
        vec![0x90, note, if on { 0x7F } else { 0x00 }]
    }

    /// Map MCU button note to control type
    pub fn button_to_control(note: u8) -> Option<McuControl> {
        match note {
            0..=7 => Some(McuControl::RecordReady(note)),
            8..=15 => Some(McuControl::Solo(note - 8)),
            16..=23 => Some(McuControl::Mute(note - 16)),
            24..=31 => Some(McuControl::Select(note - 24)),
            _ => None,
        }
    }
}

impl Default for McuProtocol {
    fn default() -> Self {
        Self::new()
    }
}

/// MCU to UCNet parameter mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McuToUcNetMapping {
    /// MCU control type
    pub mcu_control: String, // Serialized as string for storage
    /// UCNet parameter path
    pub ucnet_parameter: String,
    /// Channel number (for multi-channel controls)
    pub channel: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fader() {
        let mut mcu = McuProtocol::new();
        
        // Fader on channel 0, position 8192 (center)
        let msg = mcu.parse_midi(0xE0, 0x00, 0x40);
        assert!(matches!(msg, Some(McuMessage::Fader(_))));
        
        if let Some(McuMessage::Fader(pos)) = msg {
            assert_eq!(pos.channel, 0);
            assert_eq!(pos.position, 8192);
        }
    }

    #[test]
    fn test_parse_vpot() {
        let mut mcu = McuProtocol::new();
        
        // V-Pot 0, clockwise +1
        let msg = mcu.parse_midi(0xB0, 16, 0x01);
        assert!(matches!(msg, Some(McuMessage::VPot { .. })));
        
        if let Some(McuMessage::VPot { channel, delta }) = msg {
            assert_eq!(channel, 0);
            assert_eq!(delta, 1);
        }
    }

    #[test]
    fn test_parse_button() {
        let mut mcu = McuProtocol::new();
        
        // Mute button 0 pressed
        let msg = mcu.parse_midi(0x90, 16, 0x7F);
        assert!(matches!(msg, Some(McuMessage::Button { .. })));
        
        if let Some(McuMessage::Button { note, pressed }) = msg {
            assert_eq!(note, 16);
            assert!(pressed);
        }
    }

    #[test]
    fn test_fader_conversion() {
        assert_eq!(McuProtocol::fader_to_normalized(0), 0.0);
        assert_eq!(McuProtocol::fader_to_normalized(16383), 1.0);
        assert!((McuProtocol::fader_to_normalized(8192) - 0.5).abs() < 0.01);
        
        assert_eq!(McuProtocol::normalized_to_fader(0.0), 0);
        assert_eq!(McuProtocol::normalized_to_fader(1.0), 16383);
        assert!((McuProtocol::normalized_to_fader(0.5) as i32 - 8192).abs() < 10);
    }

    #[test]
    fn test_create_fader_update() {
        let mcu = McuProtocol::new();
        let msg = mcu.create_fader_update(0, 8192);
        assert_eq!(msg, vec![0xE0, 0x00, 0x40]);
    }

    #[test]
    fn test_create_vpot_led_update() {
        let mcu = McuProtocol::new();
        let msg = mcu.create_vpot_led_update(0, VPotLedMode::BoostCut, 6);
        assert_eq!(msg, vec![0xB0, 48, 0x16]); // (1 << 4) | 6
    }
}
