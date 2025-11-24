///! Sync integration commands
///! Wires up MIDI and UCNet events to the sync engine

use crate::commands::midi::MidiState;
use crate::commands::sync::SyncState;
use crate::midi::types::MidiMessageType;
use crate::sync::SyncEvent;
use log::{debug, error, info};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::RwLock;

/// Starts the sync integration
/// Sets up event handlers for MIDI input and UCNet changes
#[tauri::command]
pub async fn start_sync_integration(
    app_handle: AppHandle,
    midi_state: State<'_, MidiState>,
    sync_state: State<'_, SyncState>,
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

    // Set up MIDI message channel
    let mut midi_rx = {
        let mut conn_manager = midi_state
            .connection_manager
            .lock()
            .map_err(|e| format!("Failed to acquire MIDI lock: {}", e))?;
        conn_manager.setup_message_channel()
    };

    // Spawn task to handle MIDI messages
    tokio::spawn(async move {
        info!("MIDI message handler started");
        
        while let Some((device_id, message)) = midi_rx.recv().await {
            debug!("Received MIDI message from {}: {:?}", device_id, message);

            // Get engine
            let engine_lock = engine_arc.read().await;
            if let Some(engine) = engine_lock.as_ref() {
                // Handle MIDI message through sync engine
                let results = engine.handle_midi_message(device_id.clone(), message).await;

                // Emit sync events to frontend
                for result in results {
                    let event_payload = serde_json::json!({
                        "device_id": result.device_id,
                        "channel": result.channel,
                        "parameter_type": format!("{:?}", result.parameter_type),
                        "value": result.value,
                    });

                    if let Err(e) = app_handle_clone.emit("sync:parameter-synced", event_payload) {
                        error!("Failed to emit sync event: {}", e);
                    }

                    // TODO: Apply UCNet parameter changes
                    // This requires UCNet connection to be available
                    debug!(
                        "Would apply UCNet change: {} ch{} {:?} = {:?}",
                        result.device_id, result.channel, result.parameter_type, result.value
                    );
                }
            }
        }

        info!("MIDI message handler stopped");
    });

    info!("Sync integration started successfully");
    Ok(())
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
/// Useful for testing without actual MIDI hardware
#[tauri::command]
pub async fn trigger_midi_sync(
    sync_state: State<'_, SyncState>,
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
        let event_payload = serde_json::json!({
            "device_id": result.device_id,
            "channel": result.channel,
            "parameter_type": format!("{:?}", result.parameter_type),
            "value": result.value,
        });

        if let Err(e) = app_handle.emit("sync:parameter-synced", event_payload) {
            error!("Failed to emit sync event: {}", e);
        }

        applied.push(format!(
            "{} ch{} {:?} = {:?}",
            result.device_id, result.channel, result.parameter_type, result.value
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
