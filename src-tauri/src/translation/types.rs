//! Translation types for MIDI to UCNet parameter mapping

use serde::{Deserialize, Serialize};

/// UCNet parameter types that can be controlled
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UcNetParameterType {
    /// Channel volume/fader (0.0 to 1.0)
    Volume,
    /// Channel mute state (true/false)
    Mute,
    /// Channel pan position (-1.0 to 1.0)
    Pan,
}

/// UCNet parameter value
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UcNetParameterValue {
    /// Float value for continuous parameters (volume, pan)
    Float(f32),
    /// Boolean value for toggle parameters (mute)
    Bool(bool),
}

/// Taper curve type for fader response
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaperCurve {
    /// Linear response (1:1 mapping)
    Linear,
    /// Logarithmic response (for frequency-like parameters)
    Logarithmic,
    /// Audio taper (for volume faders, approximates human hearing)
    AudioTaper,
}

/// A mapping between a MIDI control and a UCNet parameter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterMapping {
    /// MIDI channel (0-15)
    pub midi_channel: u8,
    /// MIDI controller number (for CC messages)
    pub midi_controller: Option<u8>,
    /// MIDI note number (for Note On/Off messages)
    pub midi_note: Option<u8>,
    /// Target UCNet device ID
    pub ucnet_device_id: String,
    /// Target UCNet channel number (1-based, e.g., 1-32 for StudioLive 32)
    pub ucnet_channel: u32,
    /// Target parameter type
    pub parameter_type: UcNetParameterType,
    /// Taper curve for continuous parameters
    pub taper_curve: TaperCurve,
    /// Whether to use 14-bit MIDI CC (MSB/LSB pairs)
    pub use_14bit: bool,
    /// MSB controller number for 14-bit mode
    pub midi_controller_msb: Option<u8>,
    /// LSB controller number for 14-bit mode
    pub midi_controller_lsb: Option<u8>,
}

impl ParameterMapping {
    /// Creates a new volume mapping from MIDI CC to UCNet channel
    pub fn new_volume(
        midi_channel: u8,
        midi_controller: u8,
        ucnet_device_id: String,
        ucnet_channel: u32,
        taper_curve: TaperCurve,
    ) -> Self {
        Self {
            midi_channel,
            midi_controller: Some(midi_controller),
            midi_note: None,
            ucnet_device_id,
            ucnet_channel,
            parameter_type: UcNetParameterType::Volume,
            taper_curve,
            use_14bit: false,
            midi_controller_msb: None,
            midi_controller_lsb: None,
        }
    }

    /// Creates a new 14-bit volume mapping from MIDI CC MSB/LSB to UCNet channel
    pub fn new_volume_14bit(
        midi_channel: u8,
        midi_controller_msb: u8,
        midi_controller_lsb: u8,
        ucnet_device_id: String,
        ucnet_channel: u32,
        taper_curve: TaperCurve,
    ) -> Self {
        Self {
            midi_channel,
            midi_controller: None,
            midi_note: None,
            ucnet_device_id,
            ucnet_channel,
            parameter_type: UcNetParameterType::Volume,
            taper_curve,
            use_14bit: true,
            midi_controller_msb: Some(midi_controller_msb),
            midi_controller_lsb: Some(midi_controller_lsb),
        }
    }

    /// Creates a new mute mapping from MIDI Note to UCNet channel
    pub fn new_mute(
        midi_channel: u8,
        midi_note: u8,
        ucnet_device_id: String,
        ucnet_channel: u32,
    ) -> Self {
        Self {
            midi_channel,
            midi_controller: None,
            midi_note: Some(midi_note),
            ucnet_device_id,
            ucnet_channel,
            parameter_type: UcNetParameterType::Mute,
            taper_curve: TaperCurve::Linear, // Not used for boolean parameters
            use_14bit: false,
            midi_controller_msb: None,
            midi_controller_lsb: None,
        }
    }

    /// Creates a new pan mapping from MIDI CC to UCNet channel
    pub fn new_pan(
        midi_channel: u8,
        midi_controller: u8,
        ucnet_device_id: String,
        ucnet_channel: u32,
    ) -> Self {
        Self {
            midi_channel,
            midi_controller: Some(midi_controller),
            midi_note: None,
            ucnet_device_id,
            ucnet_channel,
            parameter_type: UcNetParameterType::Pan,
            taper_curve: TaperCurve::Linear,
            use_14bit: false,
            midi_controller_msb: None,
            midi_controller_lsb: None,
        }
    }

    /// Creates a new 14-bit pan mapping from MIDI CC MSB/LSB to UCNet channel
    pub fn new_pan_14bit(
        midi_channel: u8,
        midi_controller_msb: u8,
        midi_controller_lsb: u8,
        ucnet_device_id: String,
        ucnet_channel: u32,
    ) -> Self {
        Self {
            midi_channel,
            midi_controller: None,
            midi_note: None,
            ucnet_device_id,
            ucnet_channel,
            parameter_type: UcNetParameterType::Pan,
            taper_curve: TaperCurve::Linear,
            use_14bit: true,
            midi_controller_msb: Some(midi_controller_msb),
            midi_controller_lsb: Some(midi_controller_lsb),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_volume_mapping() {
        let mapping = ParameterMapping::new_volume(
            0,
            7,
            "device-1".to_string(),
            1,
            TaperCurve::AudioTaper,
        );
        
        assert_eq!(mapping.midi_channel, 0);
        assert_eq!(mapping.midi_controller, Some(7));
        assert_eq!(mapping.parameter_type, UcNetParameterType::Volume);
        assert_eq!(mapping.taper_curve, TaperCurve::AudioTaper);
        assert!(!mapping.use_14bit);
    }

    #[test]
    fn test_new_volume_14bit_mapping() {
        let mapping = ParameterMapping::new_volume_14bit(
            0,
            7,
            39,
            "device-1".to_string(),
            1,
            TaperCurve::AudioTaper,
        );
        
        assert_eq!(mapping.midi_channel, 0);
        assert_eq!(mapping.midi_controller_msb, Some(7));
        assert_eq!(mapping.midi_controller_lsb, Some(39));
        assert!(mapping.use_14bit);
    }

    #[test]
    fn test_new_mute_mapping() {
        let mapping = ParameterMapping::new_mute(0, 60, "device-1".to_string(), 1);
        
        assert_eq!(mapping.midi_channel, 0);
        assert_eq!(mapping.midi_note, Some(60));
        assert_eq!(mapping.parameter_type, UcNetParameterType::Mute);
    }

    #[test]
    fn test_new_pan_mapping() {
        let mapping = ParameterMapping::new_pan(0, 10, "device-1".to_string(), 1);
        
        assert_eq!(mapping.midi_channel, 0);
        assert_eq!(mapping.midi_controller, Some(10));
        assert_eq!(mapping.parameter_type, UcNetParameterType::Pan);
        assert_eq!(mapping.taper_curve, TaperCurve::Linear);
    }
}
