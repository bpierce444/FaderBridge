//! MIDI Learn functionality for automatic parameter mapping
//!
//! This module provides a state machine for MIDI Learn mode, allowing users to
//! quickly map MIDI controllers to mixer parameters by clicking a parameter and
//! moving a physical control.

use crate::midi::types::MidiMessageType;
use crate::translation::types::{ParameterMapping, TaperCurve, UcNetParameterType};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// MIDI Learn timeout duration (10 seconds)
const LEARN_TIMEOUT: Duration = Duration::from_secs(10);

/// MIDI Learn state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LearnState {
    /// Not in learn mode
    Idle,
    /// Waiting for MIDI input for a specific parameter
    Listening {
        /// UCNet device ID
        device_id: String,
        /// UCNet channel number
        channel: u32,
        /// Parameter type to learn
        parameter_type: UcNetParameterType,
        /// When the learn mode started
        started_at: Instant,
    },
}

/// MIDI Learn result
#[derive(Debug, Clone, PartialEq)]
pub enum LearnResult {
    /// Successfully learned a mapping
    Success(ParameterMapping),
    /// Learn mode timed out
    Timeout,
    /// Learn mode was cancelled
    Cancelled,
    /// Still waiting for MIDI input
    Waiting,
}

/// MIDI Learn engine
pub struct MidiLearn {
    state: Arc<Mutex<LearnState>>,
}

impl MidiLearn {
    /// Creates a new MIDI Learn engine
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(LearnState::Idle)),
        }
    }

    /// Starts MIDI Learn mode for a specific parameter
    ///
    /// # Arguments
    /// * `device_id` - UCNet device ID
    /// * `channel` - UCNet channel number
    /// * `parameter_type` - Type of parameter to learn
    ///
    /// # Returns
    /// `true` if learn mode was started, `false` if already in learn mode
    pub fn start_learn(
        &self,
        device_id: String,
        channel: u32,
        parameter_type: UcNetParameterType,
    ) -> bool {
        let mut state = self.state.lock().expect("Failed to lock state");
        
        if *state != LearnState::Idle {
            return false;
        }

        *state = LearnState::Listening {
            device_id,
            channel,
            parameter_type,
            started_at: Instant::now(),
        };

        true
    }

    /// Cancels MIDI Learn mode
    pub fn cancel_learn(&self) {
        let mut state = self.state.lock().expect("Failed to lock state");
        *state = LearnState::Idle;
    }

    /// Processes a MIDI message during learn mode
    ///
    /// # Arguments
    /// * `message` - MIDI message to process
    ///
    /// # Returns
    /// `LearnResult` indicating the outcome
    pub fn process_message(&self, message: &MidiMessageType) -> LearnResult {
        let mut state = self.state.lock().expect("Failed to lock state");

        match &*state {
            LearnState::Idle => LearnResult::Waiting,
            LearnState::Listening {
                device_id,
                channel,
                parameter_type,
                started_at,
            } => {
                // Check for timeout
                if started_at.elapsed() > LEARN_TIMEOUT {
                    *state = LearnState::Idle;
                    return LearnResult::Timeout;
                }

                // Filter out unwanted MIDI messages
                if !Self::is_learnable_message(message) {
                    return LearnResult::Waiting;
                }

                // Create mapping based on message type and parameter type
                let mapping = Self::create_mapping(
                    message,
                    device_id.clone(),
                    *channel,
                    *parameter_type,
                );

                // Exit learn mode
                *state = LearnState::Idle;

                match mapping {
                    Some(m) => LearnResult::Success(m),
                    None => LearnResult::Waiting,
                }
            }
        }
    }

    /// Checks if currently in learn mode
    pub fn is_learning(&self) -> bool {
        let state = self.state.lock().expect("Failed to lock state");
        !matches!(*state, LearnState::Idle)
    }

    /// Gets the current learn state
    pub fn get_state(&self) -> LearnState {
        let state = self.state.lock().expect("Failed to lock state");
        state.clone()
    }

    /// Checks if a MIDI message is learnable (filters out system messages)
    fn is_learnable_message(message: &MidiMessageType) -> bool {
        match message {
            MidiMessageType::ControlChange { .. } => true,
            MidiMessageType::NoteOn { velocity, .. } => *velocity > 0,
            MidiMessageType::NoteOff { .. } => true,
            MidiMessageType::PitchBend { .. } => true,
            MidiMessageType::ProgramChange { .. } => false, // Not useful for parameter control
        }
    }

    /// Creates a parameter mapping from a MIDI message
    fn create_mapping(
        message: &MidiMessageType,
        device_id: String,
        channel: u32,
        parameter_type: UcNetParameterType,
    ) -> Option<ParameterMapping> {
        match message {
            MidiMessageType::ControlChange {
                channel: midi_channel,
                controller,
                ..
            } => {
                // Use appropriate taper curve based on parameter type
                let taper_curve = match parameter_type {
                    UcNetParameterType::Volume => TaperCurve::AudioTaper,
                    UcNetParameterType::Pan => TaperCurve::Linear,
                    UcNetParameterType::Mute => TaperCurve::Linear,
                };

                Some(ParameterMapping {
                    midi_channel: *midi_channel,
                    midi_controller: Some(*controller),
                    midi_note: None,
                    ucnet_device_id: device_id,
                    ucnet_channel: channel,
                    parameter_type,
                    taper_curve,
                    use_14bit: false,
                    midi_controller_msb: None,
                    midi_controller_lsb: None,
                })
            }
            MidiMessageType::NoteOn {
                channel: midi_channel,
                note,
                ..
            }
            | MidiMessageType::NoteOff {
                channel: midi_channel,
                note,
                ..
            } => {
                // Notes are typically used for mute/solo buttons
                Some(ParameterMapping {
                    midi_channel: *midi_channel,
                    midi_controller: None,
                    midi_note: Some(*note),
                    ucnet_device_id: device_id,
                    ucnet_channel: channel,
                    parameter_type,
                    taper_curve: TaperCurve::Linear,
                    use_14bit: false,
                    midi_controller_msb: None,
                    midi_controller_lsb: None,
                })
            }
            MidiMessageType::PitchBend {
                channel: midi_channel,
                ..
            } => {
                // Pitch bend can be used for continuous parameters
                // We'll treat it as a special CC controller (128)
                let taper_curve = match parameter_type {
                    UcNetParameterType::Volume => TaperCurve::AudioTaper,
                    UcNetParameterType::Pan => TaperCurve::Linear,
                    UcNetParameterType::Mute => TaperCurve::Linear,
                };

                Some(ParameterMapping {
                    midi_channel: *midi_channel,
                    midi_controller: Some(128), // Special value for pitch bend
                    midi_note: None,
                    ucnet_device_id: device_id,
                    ucnet_channel: channel,
                    parameter_type,
                    taper_curve,
                    use_14bit: false,
                    midi_controller_msb: None,
                    midi_controller_lsb: None,
                })
            }
            _ => None,
        }
    }
}

impl Default for MidiLearn {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_learn() {
        let learn = MidiLearn::new();
        assert!(!learn.is_learning());

        let result = learn.start_learn(
            "device-1".to_string(),
            1,
            UcNetParameterType::Volume,
        );
        assert!(result);
        assert!(learn.is_learning());
    }

    #[test]
    fn test_cannot_start_learn_twice() {
        let learn = MidiLearn::new();
        
        let result1 = learn.start_learn(
            "device-1".to_string(),
            1,
            UcNetParameterType::Volume,
        );
        assert!(result1);

        let result2 = learn.start_learn(
            "device-2".to_string(),
            2,
            UcNetParameterType::Mute,
        );
        assert!(!result2);
    }

    #[test]
    fn test_cancel_learn() {
        let learn = MidiLearn::new();
        
        learn.start_learn("device-1".to_string(), 1, UcNetParameterType::Volume);
        assert!(learn.is_learning());

        learn.cancel_learn();
        assert!(!learn.is_learning());
    }

    #[test]
    fn test_learn_from_control_change() {
        let learn = MidiLearn::new();
        
        learn.start_learn("device-1".to_string(), 1, UcNetParameterType::Volume);

        let message = MidiMessageType::ControlChange {
            channel: 0,
            controller: 7,
            value: 64,
        };

        let result = learn.process_message(&message);
        
        match result {
            LearnResult::Success(mapping) => {
                assert_eq!(mapping.midi_channel, 0);
                assert_eq!(mapping.midi_controller, Some(7));
                assert_eq!(mapping.ucnet_device_id, "device-1");
                assert_eq!(mapping.ucnet_channel, 1);
                assert_eq!(mapping.parameter_type, UcNetParameterType::Volume);
                assert_eq!(mapping.taper_curve, TaperCurve::AudioTaper);
            }
            _ => panic!("Expected LearnResult::Success"),
        }

        assert!(!learn.is_learning());
    }

    #[test]
    fn test_learn_from_note_on() {
        let learn = MidiLearn::new();
        
        learn.start_learn("device-1".to_string(), 2, UcNetParameterType::Mute);

        let message = MidiMessageType::NoteOn {
            channel: 0,
            note: 60,
            velocity: 127,
        };

        let result = learn.process_message(&message);
        
        match result {
            LearnResult::Success(mapping) => {
                assert_eq!(mapping.midi_channel, 0);
                assert_eq!(mapping.midi_note, Some(60));
                assert_eq!(mapping.ucnet_device_id, "device-1");
                assert_eq!(mapping.ucnet_channel, 2);
                assert_eq!(mapping.parameter_type, UcNetParameterType::Mute);
            }
            _ => panic!("Expected LearnResult::Success"),
        }
    }

    #[test]
    fn test_learn_from_pitch_bend() {
        let learn = MidiLearn::new();
        
        learn.start_learn("device-1".to_string(), 3, UcNetParameterType::Pan);

        let message = MidiMessageType::PitchBend {
            channel: 0,
            value: 8192,
        };

        let result = learn.process_message(&message);
        
        match result {
            LearnResult::Success(mapping) => {
                assert_eq!(mapping.midi_channel, 0);
                assert_eq!(mapping.midi_controller, Some(128)); // Special pitch bend value
                assert_eq!(mapping.ucnet_device_id, "device-1");
                assert_eq!(mapping.ucnet_channel, 3);
                assert_eq!(mapping.parameter_type, UcNetParameterType::Pan);
            }
            _ => panic!("Expected LearnResult::Success"),
        }
    }

    #[test]
    fn test_filter_program_change() {
        let learn = MidiLearn::new();
        
        learn.start_learn("device-1".to_string(), 1, UcNetParameterType::Volume);

        let message = MidiMessageType::ProgramChange {
            channel: 0,
            program: 10,
        };

        let result = learn.process_message(&message);
        assert_eq!(result, LearnResult::Waiting);
        assert!(learn.is_learning()); // Still in learn mode
    }

    #[test]
    fn test_filter_note_on_zero_velocity() {
        let learn = MidiLearn::new();
        
        learn.start_learn("device-1".to_string(), 1, UcNetParameterType::Mute);

        let message = MidiMessageType::NoteOn {
            channel: 0,
            note: 60,
            velocity: 0, // Zero velocity = Note Off
        };

        let result = learn.process_message(&message);
        assert_eq!(result, LearnResult::Waiting);
        assert!(learn.is_learning());
    }

    #[test]
    fn test_process_message_when_idle() {
        let learn = MidiLearn::new();

        let message = MidiMessageType::ControlChange {
            channel: 0,
            controller: 7,
            value: 64,
        };

        let result = learn.process_message(&message);
        assert_eq!(result, LearnResult::Waiting);
    }

    #[test]
    fn test_get_state() {
        let learn = MidiLearn::new();
        
        let state = learn.get_state();
        assert_eq!(state, LearnState::Idle);

        learn.start_learn("device-1".to_string(), 1, UcNetParameterType::Volume);
        
        let state = learn.get_state();
        match state {
            LearnState::Listening { device_id, channel, parameter_type, .. } => {
                assert_eq!(device_id, "device-1");
                assert_eq!(channel, 1);
                assert_eq!(parameter_type, UcNetParameterType::Volume);
            }
            _ => panic!("Expected LearnState::Listening"),
        }
    }
}
