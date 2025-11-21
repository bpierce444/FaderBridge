//! Parameter mapper for translating MIDI messages to UCNet parameters

use crate::midi::types::MidiMessageType;
use super::taper::{apply_taper, midi_7bit_to_normalized, midi_14bit_to_normalized};
use super::types::{ParameterMapping, UcNetParameterType, UcNetParameterValue};

/// Result of a parameter mapping operation
#[derive(Debug, Clone, PartialEq)]
pub struct MappingResult {
    /// Target UCNet device ID
    pub device_id: String,
    /// Target channel number
    pub channel: u32,
    /// Parameter type being controlled
    pub parameter_type: UcNetParameterType,
    /// Mapped parameter value
    pub value: UcNetParameterValue,
}

/// Parameter mapper that translates MIDI messages to UCNet parameters
pub struct ParameterMapper {
    /// Active parameter mappings
    mappings: Vec<ParameterMapping>,
    /// Cache for 14-bit MIDI CC MSB values (channel, controller) -> value
    msb_cache: std::collections::HashMap<(u8, u8), u8>,
}

impl ParameterMapper {
    /// Creates a new parameter mapper
    pub fn new() -> Self {
        Self {
            mappings: Vec::new(),
            msb_cache: std::collections::HashMap::new(),
        }
    }

    /// Adds a parameter mapping
    pub fn add_mapping(&mut self, mapping: ParameterMapping) {
        self.mappings.push(mapping);
    }

    /// Removes all mappings for a specific MIDI controller
    pub fn remove_mapping(&mut self, midi_channel: u8, midi_controller: Option<u8>, midi_note: Option<u8>) {
        self.mappings.retain(|m| {
            !(m.midi_channel == midi_channel 
              && m.midi_controller == midi_controller 
              && m.midi_note == midi_note)
        });
    }

    /// Clears all mappings
    pub fn clear_mappings(&mut self) {
        self.mappings.clear();
        self.msb_cache.clear();
    }

    /// Gets all current mappings
    pub fn get_mappings(&self) -> &[ParameterMapping] {
        &self.mappings
    }

    /// Translates a MIDI message to UCNet parameter changes
    ///
    /// Returns a vector of mapping results (may be empty if no mappings match)
    pub fn translate(&mut self, message: MidiMessageType) -> Vec<MappingResult> {
        match message {
            MidiMessageType::ControlChange { channel, controller, value } => {
                self.translate_cc(channel, controller, value)
            }
            MidiMessageType::NoteOn { channel, note, velocity } => {
                self.translate_note(channel, note, velocity > 0)
            }
            MidiMessageType::NoteOff { channel, note, .. } => {
                self.translate_note(channel, note, false)
            }
            _ => Vec::new(), // Other message types not supported yet
        }
    }

    /// Translates a Control Change message
    fn translate_cc(&mut self, channel: u8, controller: u8, value: u8) -> Vec<MappingResult> {
        let mut results = Vec::new();

        for mapping in &self.mappings {
            if mapping.midi_channel != channel {
                continue;
            }

            // Handle 14-bit CC
            if mapping.use_14bit {
                if let (Some(msb_cc), Some(lsb_cc)) = (mapping.midi_controller_msb, mapping.midi_controller_lsb) {
                    if controller == msb_cc {
                        // Store MSB for later
                        self.msb_cache.insert((channel, msb_cc), value);
                        continue;
                    } else if controller == lsb_cc {
                        // Get MSB from cache
                        let msb = self.msb_cache.get(&(channel, msb_cc)).copied().unwrap_or(0);
                        let normalized = midi_14bit_to_normalized(msb, value);
                        
                        if let Some(result) = self.map_continuous_parameter(mapping, normalized) {
                            results.push(result);
                        }
                    }
                }
            } else if mapping.midi_controller == Some(controller) {
                // Handle 7-bit CC
                let normalized = midi_7bit_to_normalized(value);
                
                if let Some(result) = self.map_continuous_parameter(mapping, normalized) {
                    results.push(result);
                }
            }
        }

        results
    }

    /// Translates a Note On/Off message
    fn translate_note(&self, channel: u8, note: u8, is_on: bool) -> Vec<MappingResult> {
        let mut results = Vec::new();

        for mapping in &self.mappings {
            if mapping.midi_channel == channel 
               && mapping.midi_note == Some(note)
               && mapping.parameter_type == UcNetParameterType::Mute {
                results.push(MappingResult {
                    device_id: mapping.ucnet_device_id.clone(),
                    channel: mapping.ucnet_channel,
                    parameter_type: UcNetParameterType::Mute,
                    value: UcNetParameterValue::Bool(is_on),
                });
            }
        }

        results
    }

    /// Maps a normalized value to a continuous parameter (Volume or Pan)
    fn map_continuous_parameter(&self, mapping: &ParameterMapping, normalized: f32) -> Option<MappingResult> {
        match mapping.parameter_type {
            UcNetParameterType::Volume => {
                // Apply taper curve
                let tapered = apply_taper(normalized, mapping.taper_curve);
                Some(MappingResult {
                    device_id: mapping.ucnet_device_id.clone(),
                    channel: mapping.ucnet_channel,
                    parameter_type: UcNetParameterType::Volume,
                    value: UcNetParameterValue::Float(tapered),
                })
            }
            UcNetParameterType::Pan => {
                // Pan is -1.0 to 1.0, so we need to remap from 0.0-1.0
                let pan_value = (normalized * 2.0) - 1.0;
                Some(MappingResult {
                    device_id: mapping.ucnet_device_id.clone(),
                    channel: mapping.ucnet_channel,
                    parameter_type: UcNetParameterType::Pan,
                    value: UcNetParameterValue::Float(pan_value),
                })
            }
            UcNetParameterType::Mute => None, // Mute is handled by Note messages
        }
    }
}

impl Default for ParameterMapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::translation::types::TaperCurve;

    #[test]
    fn test_translate_volume_cc() {
        let mut mapper = ParameterMapper::new();
        
        let mapping = ParameterMapping::new_volume(
            0,
            7,
            "device-1".to_string(),
            1,
            TaperCurve::Linear,
        );
        mapper.add_mapping(mapping);

        let message = MidiMessageType::ControlChange {
            channel: 0,
            controller: 7,
            value: 64,
        };

        let results = mapper.translate(message);
        assert_eq!(results.len(), 1);
        
        let result = &results[0];
        assert_eq!(result.device_id, "device-1");
        assert_eq!(result.channel, 1);
        assert_eq!(result.parameter_type, UcNetParameterType::Volume);
        
        if let UcNetParameterValue::Float(value) = result.value {
            assert!((value - 0.504).abs() < 0.01); // 64/127 ≈ 0.504
        } else {
            panic!("Expected Float value");
        }
    }

    #[test]
    fn test_translate_volume_with_audio_taper() {
        let mut mapper = ParameterMapper::new();
        
        let mapping = ParameterMapping::new_volume(
            0,
            7,
            "device-1".to_string(),
            1,
            TaperCurve::AudioTaper,
        );
        mapper.add_mapping(mapping);

        let message = MidiMessageType::ControlChange {
            channel: 0,
            controller: 7,
            value: 64,
        };

        let results = mapper.translate(message);
        assert_eq!(results.len(), 1);
        
        if let UcNetParameterValue::Float(value) = results[0].value {
            // With audio taper, 0.504^2.5 ≈ 0.18
            assert!(value < 0.504); // Should be less than linear
            assert!(value > 0.0);
        } else {
            panic!("Expected Float value");
        }
    }

    #[test]
    fn test_translate_pan_cc() {
        let mut mapper = ParameterMapper::new();
        
        let mapping = ParameterMapping::new_pan(0, 10, "device-1".to_string(), 1);
        mapper.add_mapping(mapping);

        // Test center position
        let message = MidiMessageType::ControlChange {
            channel: 0,
            controller: 10,
            value: 64,
        };

        let results = mapper.translate(message);
        assert_eq!(results.len(), 1);
        
        if let UcNetParameterValue::Float(value) = results[0].value {
            assert!((value - 0.0).abs() < 0.05); // Should be near center (0.0)
        } else {
            panic!("Expected Float value");
        }

        // Test full left
        let message_left = MidiMessageType::ControlChange {
            channel: 0,
            controller: 10,
            value: 0,
        };
        let results_left = mapper.translate(message_left);
        if let UcNetParameterValue::Float(value) = results_left[0].value {
            assert!((value - (-1.0)).abs() < 0.01); // Should be -1.0
        }

        // Test full right
        let message_right = MidiMessageType::ControlChange {
            channel: 0,
            controller: 10,
            value: 127,
        };
        let results_right = mapper.translate(message_right);
        if let UcNetParameterValue::Float(value) = results_right[0].value {
            assert!((value - 1.0).abs() < 0.01); // Should be 1.0
        }
    }

    #[test]
    fn test_translate_mute_note() {
        let mut mapper = ParameterMapper::new();
        
        let mapping = ParameterMapping::new_mute(0, 60, "device-1".to_string(), 1);
        mapper.add_mapping(mapping);

        // Test Note On (mute on)
        let message_on = MidiMessageType::NoteOn {
            channel: 0,
            note: 60,
            velocity: 127,
        };

        let results = mapper.translate(message_on);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].parameter_type, UcNetParameterType::Mute);
        assert_eq!(results[0].value, UcNetParameterValue::Bool(true));

        // Test Note Off (mute off)
        let message_off = MidiMessageType::NoteOff {
            channel: 0,
            note: 60,
            velocity: 0,
        };

        let results = mapper.translate(message_off);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].value, UcNetParameterValue::Bool(false));
    }

    #[test]
    fn test_translate_14bit_volume() {
        let mut mapper = ParameterMapper::new();
        
        let mapping = ParameterMapping::new_volume_14bit(
            0,
            7,
            39,
            "device-1".to_string(),
            1,
            TaperCurve::Linear,
        );
        mapper.add_mapping(mapping);

        // Send MSB first
        let message_msb = MidiMessageType::ControlChange {
            channel: 0,
            controller: 7,
            value: 64,
        };
        let results_msb = mapper.translate(message_msb);
        assert_eq!(results_msb.len(), 0); // No result yet, waiting for LSB

        // Send LSB
        let message_lsb = MidiMessageType::ControlChange {
            channel: 0,
            controller: 39,
            value: 0,
        };
        let results_lsb = mapper.translate(message_lsb);
        assert_eq!(results_lsb.len(), 1);
        
        if let UcNetParameterValue::Float(value) = results_lsb[0].value {
            // 14-bit value: (64 << 7) | 0 = 8192, normalized = 8192/16383 ≈ 0.5
            assert!((value - 0.5).abs() < 0.01);
        } else {
            panic!("Expected Float value");
        }
    }

    #[test]
    fn test_multiple_mappings() {
        let mut mapper = ParameterMapper::new();
        
        // Add two mappings for different channels
        mapper.add_mapping(ParameterMapping::new_volume(
            0, 7, "device-1".to_string(), 1, TaperCurve::Linear,
        ));
        mapper.add_mapping(ParameterMapping::new_volume(
            0, 7, "device-1".to_string(), 2, TaperCurve::Linear,
        ));

        let message = MidiMessageType::ControlChange {
            channel: 0,
            controller: 7,
            value: 100,
        };

        let results = mapper.translate(message);
        assert_eq!(results.len(), 2); // Both mappings should trigger
        assert_eq!(results[0].channel, 1);
        assert_eq!(results[1].channel, 2);
    }

    #[test]
    fn test_remove_mapping() {
        let mut mapper = ParameterMapper::new();
        
        mapper.add_mapping(ParameterMapping::new_volume(
            0, 7, "device-1".to_string(), 1, TaperCurve::Linear,
        ));
        
        assert_eq!(mapper.get_mappings().len(), 1);
        
        mapper.remove_mapping(0, Some(7), None);
        assert_eq!(mapper.get_mappings().len(), 0);
    }

    #[test]
    fn test_clear_mappings() {
        let mut mapper = ParameterMapper::new();
        
        mapper.add_mapping(ParameterMapping::new_volume(
            0, 7, "device-1".to_string(), 1, TaperCurve::Linear,
        ));
        mapper.add_mapping(ParameterMapping::new_pan(
            0, 10, "device-1".to_string(), 1,
        ));
        
        assert_eq!(mapper.get_mappings().len(), 2);
        
        mapper.clear_mappings();
        assert_eq!(mapper.get_mappings().len(), 0);
    }

    #[test]
    fn test_no_matching_mapping() {
        let mut mapper = ParameterMapper::new();
        
        mapper.add_mapping(ParameterMapping::new_volume(
            0, 7, "device-1".to_string(), 1, TaperCurve::Linear,
        ));

        // Send message on different controller
        let message = MidiMessageType::ControlChange {
            channel: 0,
            controller: 8,
            value: 64,
        };

        let results = mapper.translate(message);
        assert_eq!(results.len(), 0);
    }
}
