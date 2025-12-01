//! Sync integration commands
//!
//! Wires up MIDI and UCNet events to the sync engine, enabling bidirectional
//! synchronization between MIDI controllers and UCNet mixers.

use crate::commands::midi::MidiState;
use crate::commands::sync::SyncState;
use crate::commands::ucnet::UcNetState;
use crate::midi::types::MidiMessageType;
use crate::translation::types::{UcNetParameterType, UcNetParameterValue};
use log::{debug, error, info, warn};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

/// Starts the sync integration
///
/// Sets up event handlers for MIDI input and UCNet changes, enabling
/// bidirectional synchronization between MIDI controllers and UCNet mixers.
///
/// # Arguments
/// * `app_handle` - Tauri app handle for emitting events
/// * `midi_state` - MIDI connection state
/// * `sync_state` - Sync engine state
/// * `ucnet_state` - UCNet connection state
#[tauri::command]
pub async fn start_sync_integration(
    app_handle: AppHandle,
    midi_state: State<'_, MidiState>,
    sync_state: State<'_, SyncState>,
    ucnet_state: State<'_, UcNetState>,
) -> Result<(), String> {
    info!("Starting sync integration...");

    // Check that sync engine is initialized
    {
        let engine_lock = sync_state.engine.read().await;
        if engine_lock.is_none() {
            return Err("Sync engine not initialized".to_string());
        }
    }

    // Clone the Arc to the engine for the async task
    let engine_arc = sync_state.engine.clone();
    let app_handle_clone = app_handle.clone();
    let ucnet_connection_manager = Arc::clone(&ucnet_state.connection_manager);

    // Set up MIDI message channel
    let mut midi_rx = {
        let mut conn_manager = midi_state
            .connection_manager
            .lock()
            .map_err(|e| format!("Failed to acquire MIDI lock: {}", e))?;
        conn_manager.setup_message_channel()
    };

    // Spawn task to handle MIDI messages and apply UCNet changes
    tokio::spawn(async move {
        info!("MIDI→UCNet sync handler started");
        
        while let Some((device_id, message)) = midi_rx.recv().await {
            debug!("Received MIDI message from {}: {:?}", device_id, message);

            // Emit raw MIDI message event for Message Monitor
            let midi_event_payload = match &message {
                MidiMessageType::ControlChange { channel, controller, value } => {
                    serde_json::json!({
                        "device_id": device_id,
                        "type": "control_change",
                        "channel": channel,
                        "controller": controller,
                        "value": value,
                    })
                }
                MidiMessageType::NoteOn { channel, note, velocity } => {
                    serde_json::json!({
                        "device_id": device_id,
                        "type": "note_on",
                        "channel": channel,
                        "note": note,
                        "velocity": velocity,
                    })
                }
                MidiMessageType::NoteOff { channel, note, velocity } => {
                    serde_json::json!({
                        "device_id": device_id,
                        "type": "note_off",
                        "channel": channel,
                        "note": note,
                        "velocity": velocity,
                    })
                }
                MidiMessageType::PitchBend { channel, value } => {
                    serde_json::json!({
                        "device_id": device_id,
                        "type": "pitch_bend",
                        "channel": channel,
                        "value": value,
                    })
                }
                MidiMessageType::ProgramChange { channel, program } => {
                    serde_json::json!({
                        "device_id": device_id,
                        "type": "program_change",
                        "channel": channel,
                        "program": program,
                    })
                }
            };

            if let Err(e) = app_handle_clone.emit("midi:message-received", midi_event_payload) {
                error!("Failed to emit MIDI message event: {}", e);
            }

            // Get engine
            let engine_lock = engine_arc.read().await;
            if let Some(engine) = engine_lock.as_ref() {
                // Handle MIDI message through sync engine
                let results = engine.handle_midi_message(device_id.clone(), message).await;

                // Apply UCNet parameter changes and emit events
                for result in results {
                    // Apply the parameter change to the UCNet device
                    let apply_result = apply_ucnet_parameter(
                        &ucnet_connection_manager,
                        &result.device_id,
                        result.channel,
                        result.parameter_type,
                        &result.value,
                        Some(&app_handle_clone),
                    ).await;

                    // Emit sync event to frontend (include success/failure status)
                    let event_payload = serde_json::json!({
                        "device_id": result.device_id,
                        "channel": result.channel,
                        "parameter_type": format!("{:?}", result.parameter_type),
                        "value": result.value,
                        "applied": apply_result.is_ok(),
                        "error": apply_result.as_ref().err().map(|e| e.to_string()),
                    });

                    if let Err(e) = app_handle_clone.emit("sync:parameter-synced", event_payload) {
                        error!("Failed to emit sync event: {}", e);
                    }

                    match apply_result {
                        Ok(()) => {
                            debug!(
                                "Applied UCNet change: {} ch{} {:?} = {:?}",
                                result.device_id, result.channel, result.parameter_type, result.value
                            );
                        }
                        Err(e) => {
                            warn!(
                                "Failed to apply UCNet change: {} ch{} {:?} = {:?}: {}",
                                result.device_id, result.channel, result.parameter_type, result.value, e
                            );
                        }
                    }
                }
            }
        }

        info!("MIDI→UCNet sync handler stopped");
    });

    info!("Sync integration started successfully");
    Ok(())
}

/// Applies a parameter change to a UCNet device and emits events for the Message Monitor
///
/// # Arguments
/// * `connection_manager` - UCNet connection manager
/// * `device_id` - Target UCNet device ID
/// * `channel` - Target channel number (1-based)
/// * `parameter_type` - Type of parameter to change
/// * `value` - New parameter value
/// * `app_handle` - Optional Tauri app handle for emitting events
async fn apply_ucnet_parameter(
    connection_manager: &Arc<crate::ucnet::ConnectionManager>,
    device_id: &str,
    channel: u32,
    parameter_type: UcNetParameterType,
    value: &UcNetParameterValue,
    app_handle: Option<&AppHandle>,
) -> Result<(), String> {
    // Check if device is connected
    let device_state = connection_manager.get_device_state(device_id).await;
    if device_state.is_none() {
        return Err(format!("UCNet device '{}' not connected", device_id));
    }

    // Build parameter key for logging
    let param_key = match parameter_type {
        UcNetParameterType::Volume => format!("line.ch{}.volume", channel),
        UcNetParameterType::Mute => format!("line.ch{}.mute", channel),
        UcNetParameterType::Pan => format!("line.ch{}.pan", channel),
    };

    // Emit UCNet message-sent event for Message Monitor (before sending)
    if let Some(handle) = app_handle {
        let event_payload = serde_json::json!({
            "device_id": device_id,
            "parameter": param_key,
            "channel": channel,
            "parameter_type": format!("{:?}", parameter_type),
            "value": value,
        });
        if let Err(e) = handle.emit("ucnet:message-sent", event_payload) {
            error!("Failed to emit UCNet message-sent event: {}", e);
        }
    }

    // Apply the parameter based on type
    let result = match (parameter_type, value) {
        (UcNetParameterType::Volume, UcNetParameterValue::Float(vol)) => {
            connection_manager
                .set_channel_volume(device_id, channel as u8, *vol)
                .await
                .map_err(|e| e.to_string())
        }
        (UcNetParameterType::Mute, UcNetParameterValue::Bool(muted)) => {
            connection_manager
                .set_channel_mute(device_id, channel as u8, *muted)
                .await
                .map_err(|e| e.to_string())
        }
        (UcNetParameterType::Pan, UcNetParameterValue::Float(pan)) => {
            connection_manager
                .set_channel_pan(device_id, channel as u8, *pan)
                .await
                .map_err(|e| e.to_string())
        }
        _ => Err(format!(
            "Invalid parameter type/value combination: {:?}/{:?}",
            parameter_type, value
        )),
    };

    result
}

/// Stops the sync integration
#[tauri::command]
pub async fn stop_sync_integration() -> Result<(), String> {
    info!("Stopping sync integration...");
    // The MIDI message handler will stop when the channel is closed
    // This happens automatically when MIDI devices are disconnected
    Ok(())
}

/// Manually trigger a sync from MIDI to UCNet
///
/// Useful for testing without actual MIDI hardware. This command processes
/// a MIDI message through the sync engine and applies the resulting UCNet
/// parameter changes.
///
/// # Arguments
/// * `sync_state` - Sync engine state
/// * `ucnet_state` - UCNet connection state
/// * `device_id` - MIDI device ID (source of the message)
/// * `message` - MIDI message to process
/// * `app_handle` - Tauri app handle for emitting events
#[tauri::command]
pub async fn trigger_midi_sync(
    sync_state: State<'_, SyncState>,
    ucnet_state: State<'_, UcNetState>,
    device_id: String,
    message: MidiMessageType,
    app_handle: AppHandle,
) -> Result<Vec<String>, String> {
    let engine_lock = sync_state.engine.read().await;
    let engine = engine_lock
        .as_ref()
        .ok_or("Sync engine not initialized")?;

    let results = engine.handle_midi_message(device_id, message).await;
    
    let mut applied = Vec::new();
    for result in results {
        // Apply the parameter change to the UCNet device
        let apply_result = apply_ucnet_parameter(
            &ucnet_state.connection_manager,
            &result.device_id,
            result.channel,
            result.parameter_type,
            &result.value,
            Some(&app_handle),
        ).await;

        // Emit sync event to frontend (include success/failure status)
        let event_payload = serde_json::json!({
            "device_id": result.device_id,
            "channel": result.channel,
            "parameter_type": format!("{:?}", result.parameter_type),
            "value": result.value,
            "applied": apply_result.is_ok(),
            "error": apply_result.as_ref().err().map(|e| e.to_string()),
        });

        if let Err(e) = app_handle.emit("sync:parameter-synced", event_payload) {
            error!("Failed to emit sync event: {}", e);
        }

        let status = if apply_result.is_ok() { "applied" } else { "failed" };
        applied.push(format!(
            "{} ch{} {:?} = {:?} ({})",
            result.device_id, result.channel, result.parameter_type, result.value, status
        ));
    }

    Ok(applied)
}

/// Get sync engine status
#[tauri::command]
pub async fn get_sync_status(
    sync_state: State<'_, SyncState>,
) -> Result<SyncStatusResponse, String> {
    let engine_lock = sync_state.engine.read().await;
    
    if let Some(engine) = engine_lock.as_ref() {
        let mappings = engine.get_mappings().await;
        let latency_stats = engine.get_latency_stats().await;

        Ok(SyncStatusResponse {
            initialized: true,
            mapping_count: mappings.len(),
            latency_stats: latency_stats.map(|s| LatencyStatsResponse {
                avg_ms: s.avg_ms,
                min_ms: s.min_ms,
                max_ms: s.max_ms,
                sample_count: s.sample_count,
            }),
        })
    } else {
        Ok(SyncStatusResponse {
            initialized: false,
            mapping_count: 0,
            latency_stats: None,
        })
    }
}

/// Response for sync status
#[derive(Debug, serde::Serialize)]
pub struct SyncStatusResponse {
    pub initialized: bool,
    pub mapping_count: usize,
    pub latency_stats: Option<LatencyStatsResponse>,
}

/// Latency statistics response
#[derive(Debug, serde::Serialize)]
pub struct LatencyStatsResponse {
    pub avg_ms: f64,
    pub min_ms: f64,
    pub max_ms: f64,
    pub sample_count: usize,
}

/// Manually trigger a sync from UCNet to MIDI (reverse direction)
///
/// Useful for testing without actual UCNet hardware. This command processes
/// a UCNet parameter change through the sync engine and sends the resulting
/// MIDI messages to all connected output devices.
///
/// # Arguments
/// * `sync_state` - Sync engine state
/// * `midi_state` - MIDI connection state
/// * `device_id` - UCNet device ID (source of the change)
/// * `channel` - UCNet channel number (1-based)
/// * `parameter_type` - Type of parameter that changed
/// * `value` - New parameter value
/// * `app_handle` - Tauri app handle for emitting events
#[tauri::command]
pub async fn trigger_ucnet_sync(
    sync_state: State<'_, SyncState>,
    midi_state: State<'_, MidiState>,
    device_id: String,
    channel: u32,
    parameter_type: UcNetParameterType,
    value: UcNetParameterValue,
    app_handle: AppHandle,
) -> Result<Vec<String>, String> {
    let engine_lock = sync_state.engine.read().await;
    let engine = engine_lock
        .as_ref()
        .ok_or("Sync engine not initialized")?;

    // Handle UCNet change through sync engine to get MIDI messages
    let midi_messages = engine
        .handle_ucnet_change(device_id.clone(), channel, parameter_type, value.clone())
        .await;

    if midi_messages.is_empty() {
        return Ok(vec!["No MIDI mappings found for this parameter".to_string()]);
    }

    // Send MIDI messages to all connected output devices
    let conn_manager = midi_state
        .connection_manager
        .lock()
        .map_err(|e| format!("Failed to acquire MIDI lock: {}", e))?;

    let mut results = Vec::new();
    for message in midi_messages {
        match conn_manager.broadcast_message(message.clone()) {
            Ok(count) => {
                results.push(format!(
                    "Sent {:?} to {} output device(s)",
                    message, count
                ));
                
                // Emit event to frontend
                let event_payload = serde_json::json!({
                    "direction": "ucnet_to_midi",
                    "source_device_id": device_id,
                    "channel": channel,
                    "parameter_type": format!("{:?}", parameter_type),
                    "value": value,
                    "midi_message": format!("{:?}", message),
                    "devices_sent": count,
                });

                if let Err(e) = app_handle.emit("sync:midi-sent", event_payload) {
                    error!("Failed to emit MIDI sent event: {}", e);
                }
            }
            Err(e) => {
                results.push(format!("Failed to send {:?}: {}", message, e));
            }
        }
    }

    Ok(results)
}

/// Sends MIDI messages to controllers based on UCNet parameter changes
///
/// This is a helper function used internally to send MIDI output when
/// UCNet parameters change on the mixer.
///
/// # Arguments
/// * `midi_state` - MIDI connection state (must be cloned Arc for async use)
/// * `messages` - MIDI messages to send
///
/// # Returns
/// Number of messages successfully broadcast
pub fn send_midi_output(
    midi_state: &std::sync::Mutex<crate::midi::MidiConnectionManager>,
    messages: &[MidiMessageType],
) -> Result<usize, String> {
    let conn_manager = midi_state
        .lock()
        .map_err(|e| format!("Failed to acquire MIDI lock: {}", e))?;

    let mut total_sent = 0;
    for message in messages {
        match conn_manager.broadcast_message(message.clone()) {
            Ok(count) => {
                total_sent += count;
                debug!("Broadcast MIDI message {:?} to {} devices", message, count);
            }
            Err(e) => {
                warn!("Failed to broadcast MIDI message {:?}: {}", message, e);
            }
        }
    }

    Ok(total_sent)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_status_response_serialization() {
        let response = SyncStatusResponse {
            initialized: true,
            mapping_count: 5,
            latency_stats: Some(LatencyStatsResponse {
                avg_ms: 5.5,
                min_ms: 2.0,
                max_ms: 12.0,
                sample_count: 100,
            }),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"initialized\":true"));
        assert!(json.contains("\"mapping_count\":5"));
    }
}
