//! Tauri commands for sync operations

use crate::sync::{LatencyStats, SyncEngine};
use crate::translation::types::{ParameterMapping, TaperCurve, UcNetParameterType};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

/// Global sync state
pub struct SyncState {
    pub engine: Arc<RwLock<Option<SyncEngine>>>,
}

impl SyncState {
    pub fn new() -> Self {
        Self {
            engine: Arc::new(RwLock::new(None)),
        }
    }
}

/// Response for latency stats command
#[derive(Debug, Clone, Serialize)]
pub struct LatencyStatsResponse {
    pub avg_ms: f64,
    pub min_ms: f64,
    pub max_ms: f64,
    pub sample_count: usize,
}

impl From<LatencyStats> for LatencyStatsResponse {
    fn from(stats: LatencyStats) -> Self {
        Self {
            avg_ms: stats.avg_ms,
            min_ms: stats.min_ms,
            max_ms: stats.max_ms,
            sample_count: stats.sample_count,
        }
    }
}

/// Request for adding a parameter mapping
#[derive(Debug, Clone, Deserialize)]
pub struct AddMappingRequest {
    pub midi_channel: u8,
    pub midi_controller: Option<u8>,
    pub midi_note: Option<u8>,
    pub ucnet_device_id: String,
    pub ucnet_channel: u32,
    pub parameter_type: String, // "volume", "pan", "mute"
    pub taper_curve: Option<String>, // "linear", "logarithmic", "audio"
    pub use_14bit: Option<bool>,
    pub midi_controller_msb: Option<u8>,
    pub midi_controller_lsb: Option<u8>,
}

/// Initializes the sync engine
#[tauri::command]
pub async fn init_sync_engine(state: State<'_, SyncState>) -> Result<(), String> {
    let mut engine_lock = state.engine.write().await;
    
    if engine_lock.is_some() {
        return Err("Sync engine already initialized".to_string());
    }

    let (engine, _rx) = SyncEngine::default();
    *engine_lock = Some(engine);

    Ok(())
}

/// Adds a parameter mapping
#[tauri::command]
pub async fn add_parameter_mapping(
    state: State<'_, SyncState>,
    request: AddMappingRequest,
) -> Result<(), String> {
    let engine_lock = state.engine.read().await;
    let engine = engine_lock
        .as_ref()
        .ok_or("Sync engine not initialized")?;

    // Parse parameter type
    let parameter_type = match request.parameter_type.to_lowercase().as_str() {
        "volume" => UcNetParameterType::Volume,
        "pan" => UcNetParameterType::Pan,
        "mute" => UcNetParameterType::Mute,
        _ => return Err(format!("Invalid parameter type: {}", request.parameter_type)),
    };

    // Parse taper curve (only for volume)
    let taper_curve = if parameter_type == UcNetParameterType::Volume {
        match request.taper_curve.as_deref() {
            Some("linear") | None => TaperCurve::Linear,
            Some("logarithmic") => TaperCurve::Logarithmic,
            Some("audio") => TaperCurve::AudioTaper,
            Some(curve) => return Err(format!("Invalid taper curve: {}", curve)),
        }
    } else {
        TaperCurve::Linear // Default for non-volume parameters
    };

    // Create mapping based on parameter type
    let mapping = if parameter_type == UcNetParameterType::Mute {
        // Mute uses MIDI notes
        let midi_note = request
            .midi_note
            .ok_or("MIDI note required for mute parameter")?;
        ParameterMapping::new_mute(
            request.midi_channel,
            midi_note,
            request.ucnet_device_id,
            request.ucnet_channel,
        )
    } else if request.use_14bit.unwrap_or(false) {
        // 14-bit CC mapping
        let msb = request
            .midi_controller_msb
            .ok_or("MIDI controller MSB required for 14-bit mapping")?;
        let lsb = request
            .midi_controller_lsb
            .ok_or("MIDI controller LSB required for 14-bit mapping")?;

        if parameter_type == UcNetParameterType::Volume {
            ParameterMapping::new_volume_14bit(
                request.midi_channel,
                msb,
                lsb,
                request.ucnet_device_id,
                request.ucnet_channel,
                taper_curve,
            )
        } else {
            ParameterMapping::new_pan_14bit(
                request.midi_channel,
                msb,
                lsb,
                request.ucnet_device_id,
                request.ucnet_channel,
            )
        }
    } else {
        // 7-bit CC mapping
        let controller = request
            .midi_controller
            .ok_or("MIDI controller required for CC mapping")?;

        if parameter_type == UcNetParameterType::Volume {
            ParameterMapping::new_volume(
                request.midi_channel,
                controller,
                request.ucnet_device_id,
                request.ucnet_channel,
                taper_curve,
            )
        } else {
            ParameterMapping::new_pan(
                request.midi_channel,
                controller,
                request.ucnet_device_id,
                request.ucnet_channel,
            )
        }
    };

    engine.add_mapping(mapping).await;
    Ok(())
}

/// Removes a parameter mapping
#[tauri::command]
pub async fn remove_parameter_mapping(
    state: State<'_, SyncState>,
    midi_channel: u8,
    midi_controller: Option<u8>,
    midi_note: Option<u8>,
) -> Result<(), String> {
    let engine_lock = state.engine.read().await;
    let engine = engine_lock
        .as_ref()
        .ok_or("Sync engine not initialized")?;

    engine
        .remove_mapping(midi_channel, midi_controller, midi_note)
        .await;
    Ok(())
}

/// Clears all parameter mappings
#[tauri::command]
pub async fn clear_parameter_mappings(state: State<'_, SyncState>) -> Result<(), String> {
    let engine_lock = state.engine.read().await;
    let engine = engine_lock
        .as_ref()
        .ok_or("Sync engine not initialized")?;

    engine.clear_mappings().await;
    Ok(())
}

/// Gets all parameter mappings
#[tauri::command]
pub async fn get_parameter_mappings(
    state: State<'_, SyncState>,
) -> Result<Vec<ParameterMapping>, String> {
    let engine_lock = state.engine.read().await;
    let engine = engine_lock
        .as_ref()
        .ok_or("Sync engine not initialized")?;

    Ok(engine.get_mappings().await)
}

/// Gets latency statistics
#[tauri::command]
pub async fn get_latency_stats(
    state: State<'_, SyncState>,
) -> Result<Option<LatencyStatsResponse>, String> {
    let engine_lock = state.engine.read().await;
    let engine = engine_lock
        .as_ref()
        .ok_or("Sync engine not initialized")?;

    Ok(engine.get_latency_stats().await.map(|s| s.into()))
}

/// Clears latency statistics
#[tauri::command]
pub async fn clear_latency_stats(state: State<'_, SyncState>) -> Result<(), String> {
    let engine_lock = state.engine.read().await;
    let engine = engine_lock
        .as_ref()
        .ok_or("Sync engine not initialized")?;

    engine.clear_latency_stats().await;
    Ok(())
}

/// Clears shadow state for a specific device
#[tauri::command]
pub async fn clear_device_state(
    state: State<'_, SyncState>,
    device_id: String,
) -> Result<(), String> {
    let engine_lock = state.engine.read().await;
    let engine = engine_lock
        .as_ref()
        .ok_or("Sync engine not initialized")?;

    engine.clear_device_state(&device_id).await;
    Ok(())
}

/// Clears all shadow state
#[tauri::command]
pub async fn clear_all_state(state: State<'_, SyncState>) -> Result<(), String> {
    let engine_lock = state.engine.read().await;
    let engine = engine_lock
        .as_ref()
        .ok_or("Sync engine not initialized")?;

    engine.clear_all_state().await;
    Ok(())
}
