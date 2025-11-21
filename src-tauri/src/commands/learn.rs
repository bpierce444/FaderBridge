//! Tauri commands for MIDI Learn functionality

use crate::midi::{LearnResult, LearnState, MidiLearn};
use crate::translation::types::{ParameterMapping, UcNetParameterType};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::State;

/// Serializable version of LearnState for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum LearnStateDto {
    Idle,
    Listening {
        device_id: String,
        channel: u32,
        parameter_type: UcNetParameterType,
        elapsed_ms: u128,
    },
}

impl From<LearnState> for LearnStateDto {
    fn from(state: LearnState) -> Self {
        match state {
            LearnState::Idle => LearnStateDto::Idle,
            LearnState::Listening {
                device_id,
                channel,
                parameter_type,
                started_at,
            } => LearnStateDto::Listening {
                device_id,
                channel,
                parameter_type,
                elapsed_ms: started_at.elapsed().as_millis(),
            },
        }
    }
}

/// Serializable version of LearnResult for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum LearnResultDto {
    Success { mapping: ParameterMapping },
    Timeout,
    Cancelled,
    Waiting,
}

impl From<LearnResult> for LearnResultDto {
    fn from(result: LearnResult) -> Self {
        match result {
            LearnResult::Success(mapping) => LearnResultDto::Success { mapping },
            LearnResult::Timeout => LearnResultDto::Timeout,
            LearnResult::Cancelled => LearnResultDto::Cancelled,
            LearnResult::Waiting => LearnResultDto::Waiting,
        }
    }
}

/// Global MIDI Learn instance
pub struct MidiLearnState {
    pub learn: Arc<Mutex<MidiLearn>>,
}

impl MidiLearnState {
    pub fn new() -> Self {
        Self {
            learn: Arc::new(Mutex::new(MidiLearn::new())),
        }
    }
}

impl Default for MidiLearnState {
    fn default() -> Self {
        Self::new()
    }
}

/// Starts MIDI Learn mode for a specific parameter
///
/// # Arguments
/// * `device_id` - UCNet device ID
/// * `channel` - UCNet channel number
/// * `parameter_type` - Type of parameter to learn (volume, mute, pan)
///
/// # Returns
/// `true` if learn mode was started, `false` if already in learn mode
#[tauri::command]
pub fn start_midi_learn(
    device_id: String,
    channel: u32,
    parameter_type: UcNetParameterType,
    state: State<MidiLearnState>,
) -> Result<bool, String> {
    let learn = state.learn.lock().map_err(|e| e.to_string())?;
    Ok(learn.start_learn(device_id, channel, parameter_type))
}

/// Cancels MIDI Learn mode
#[tauri::command]
pub fn cancel_midi_learn(state: State<MidiLearnState>) -> Result<(), String> {
    let learn = state.learn.lock().map_err(|e| e.to_string())?;
    learn.cancel_learn();
    Ok(())
}

/// Gets the current MIDI Learn state
#[tauri::command]
pub fn get_midi_learn_state(state: State<MidiLearnState>) -> Result<LearnStateDto, String> {
    let learn = state.learn.lock().map_err(|e| e.to_string())?;
    Ok(LearnStateDto::from(learn.get_state()))
}

/// Checks if currently in MIDI Learn mode
#[tauri::command]
pub fn is_midi_learning(state: State<MidiLearnState>) -> Result<bool, String> {
    let learn = state.learn.lock().map_err(|e| e.to_string())?;
    Ok(learn.is_learning())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::translation::types::TaperCurve;

    #[test]
    fn test_learn_state_dto_from_idle() {
        let state = LearnState::Idle;
        let dto = LearnStateDto::from(state);
        
        match dto {
            LearnStateDto::Idle => {}
            _ => panic!("Expected Idle state"),
        }
    }

    #[test]
    fn test_learn_state_dto_from_listening() {
        let state = LearnState::Listening {
            device_id: "device-1".to_string(),
            channel: 1,
            parameter_type: UcNetParameterType::Volume,
            started_at: std::time::Instant::now(),
        };
        
        let dto = LearnStateDto::from(state);
        
        match dto {
            LearnStateDto::Listening {
                device_id,
                channel,
                parameter_type,
                ..
            } => {
                assert_eq!(device_id, "device-1");
                assert_eq!(channel, 1);
                assert_eq!(parameter_type, UcNetParameterType::Volume);
            }
            _ => panic!("Expected Listening state"),
        }
    }

    #[test]
    fn test_learn_result_dto_from_success() {
        let mapping = ParameterMapping::new_volume(
            0,
            7,
            "device-1".to_string(),
            1,
            TaperCurve::AudioTaper,
        );
        
        let result = LearnResult::Success(mapping.clone());
        let dto = LearnResultDto::from(result);
        
        match dto {
            LearnResultDto::Success { mapping: m } => {
                assert_eq!(m.midi_channel, 0);
                assert_eq!(m.midi_controller, Some(7));
            }
            _ => panic!("Expected Success result"),
        }
    }

    #[test]
    fn test_learn_result_dto_from_timeout() {
        let result = LearnResult::Timeout;
        let dto = LearnResultDto::from(result);
        
        match dto {
            LearnResultDto::Timeout => {}
            _ => panic!("Expected Timeout result"),
        }
    }

    #[test]
    fn test_learn_result_dto_from_cancelled() {
        let result = LearnResult::Cancelled;
        let dto = LearnResultDto::from(result);
        
        match dto {
            LearnResultDto::Cancelled => {}
            _ => panic!("Expected Cancelled result"),
        }
    }

    #[test]
    fn test_learn_result_dto_from_waiting() {
        let result = LearnResult::Waiting;
        let dto = LearnResultDto::from(result);
        
        match dto {
            LearnResultDto::Waiting => {}
            _ => panic!("Expected Waiting result"),
        }
    }

    #[test]
    fn test_midi_learn_state_new() {
        let state = MidiLearnState::new();
        let learn = state.learn.lock().unwrap();
        assert!(!learn.is_learning());
    }
}
