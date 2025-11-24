//! Bidirectional synchronization engine
//!
//! Orchestrates synchronization between MIDI controllers and UCNet devices with
//! < 10ms latency and feedback loop prevention.

use super::shadow_state::{ParameterId, ShadowState};
use crate::midi::types::MidiMessageType;
use crate::translation::mapper::{MappingResult, ParameterMapper};
use crate::translation::types::{ParameterMapping, UcNetParameterValue};
use log::{debug, warn};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};

/// Sync event types for event-driven architecture
#[derive(Debug, Clone)]
pub enum SyncEvent {
    /// MIDI message received from controller
    MidiReceived {
        device_id: String,
        message: MidiMessageType,
        timestamp: Instant,
    },
    /// UCNet parameter changed on mixer
    UcNetChanged {
        device_id: String,
        channel: u32,
        parameter_type: crate::translation::types::UcNetParameterType,
        value: UcNetParameterValue,
        timestamp: Instant,
    },
    /// Parameter synchronized (for monitoring/logging)
    ParameterSynced {
        source: SyncSource,
        device_id: String,
        channel: u32,
        parameter_type: crate::translation::types::UcNetParameterType,
        latency_ms: f64,
    },
}

/// Source of a sync event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncSource {
    /// Event originated from MIDI controller
    Midi,
    /// Event originated from UCNet mixer
    UcNet,
}

/// Latency statistics for monitoring performance
#[derive(Debug, Clone)]
pub struct LatencyStats {
    /// Average latency in milliseconds
    pub avg_ms: f64,
    /// Minimum latency in milliseconds
    pub min_ms: f64,
    /// Maximum latency in milliseconds
    pub max_ms: f64,
    /// Number of samples
    pub sample_count: usize,
}

/// Bidirectional synchronization engine
pub struct SyncEngine {
    /// Parameter mapper for MIDI → UCNet translation
    mapper: Arc<RwLock<ParameterMapper>>,
    /// Shadow state for feedback loop prevention
    shadow_state: Arc<RwLock<ShadowState>>,
    /// Event sender for sync events
    event_tx: mpsc::UnboundedSender<SyncEvent>,
    /// Latency measurements (in milliseconds)
    latency_samples: Arc<RwLock<Vec<f64>>>,
    /// Maximum number of latency samples to keep
    max_latency_samples: usize,
}

impl SyncEngine {
    /// Creates a new sync engine
    ///
    /// # Arguments
    /// * `max_latency_samples` - Maximum number of latency samples to keep (default: 1000)
    pub fn new(max_latency_samples: usize) -> (Self, mpsc::UnboundedReceiver<SyncEvent>) {
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        let engine = Self {
            mapper: Arc::new(RwLock::new(ParameterMapper::new())),
            shadow_state: Arc::new(RwLock::new(ShadowState::default())),
            event_tx,
            latency_samples: Arc::new(RwLock::new(Vec::new())),
            max_latency_samples,
        };

        (engine, event_rx)
    }

    /// Creates a sync engine with default settings
    pub fn default() -> (Self, mpsc::UnboundedReceiver<SyncEvent>) {
        Self::new(1000)
    }

    /// Adds a parameter mapping
    pub async fn add_mapping(&self, mapping: ParameterMapping) {
        let mut mapper = self.mapper.write().await;
        mapper.add_mapping(mapping);
    }

    /// Removes a parameter mapping
    pub async fn remove_mapping(
        &self,
        midi_channel: u8,
        midi_controller: Option<u8>,
        midi_note: Option<u8>,
    ) {
        let mut mapper = self.mapper.write().await;
        mapper.remove_mapping(midi_channel, midi_controller, midi_note);
    }

    /// Clears all parameter mappings
    pub async fn clear_mappings(&self) {
        let mut mapper = self.mapper.write().await;
        mapper.clear_mappings();
    }

    /// Gets all current mappings
    pub async fn get_mappings(&self) -> Vec<ParameterMapping> {
        let mapper = self.mapper.read().await;
        mapper.get_mappings().to_vec()
    }

    /// Handles a MIDI message from a controller
    ///
    /// Translates the MIDI message to UCNet parameters and updates shadow state.
    /// Returns a vector of UCNet parameter changes to apply.
    ///
    /// # Arguments
    /// * `device_id` - MIDI device ID
    /// * `message` - MIDI message
    pub async fn handle_midi_message(
        &self,
        device_id: String,
        message: MidiMessageType,
    ) -> Vec<MappingResult> {
        let start_time = Instant::now();

        // Translate MIDI to UCNet parameters
        let mut mapper = self.mapper.write().await;
        let results = mapper.translate(message.clone());
        drop(mapper); // Release lock early

        // Filter results through shadow state to prevent feedback loops
        let mut filtered_results = Vec::new();
        let mut shadow = self.shadow_state.write().await;

        for result in results {
            let param_id = ParameterId::new(
                result.device_id.clone(),
                result.channel,
                result.parameter_type,
            );

            // Check if value has changed
            if shadow.has_changed(&param_id, &result.value) {
                // Update shadow state
                shadow.update(param_id.clone(), result.value.clone());
                filtered_results.push(result.clone());

                // Measure latency
                let latency_ms = start_time.elapsed().as_secs_f64() * 1000.0;
                self.record_latency(latency_ms).await;

                // Emit sync event
                let _ = self.event_tx.send(SyncEvent::ParameterSynced {
                    source: SyncSource::Midi,
                    device_id: result.device_id.clone(),
                    channel: result.channel,
                    parameter_type: result.parameter_type,
                    latency_ms,
                });

                debug!(
                    "MIDI→UCNet: {} ch{} {:?} = {:?} ({:.2}ms)",
                    result.device_id, result.channel, result.parameter_type, result.value, latency_ms
                );
            } else {
                debug!(
                    "Filtered duplicate: {} ch{} {:?}",
                    result.device_id, result.channel, result.parameter_type
                );
            }
        }

        // Emit MIDI received event
        let _ = self.event_tx.send(SyncEvent::MidiReceived {
            device_id,
            message,
            timestamp: start_time,
        });

        filtered_results
    }

    /// Handles a UCNet parameter change from the mixer
    ///
    /// Updates shadow state and returns MIDI messages to send to controllers.
    ///
    /// # Arguments
    /// * `device_id` - UCNet device ID
    /// * `channel` - Channel number
    /// * `parameter_type` - Parameter type
    /// * `value` - New parameter value
    pub async fn handle_ucnet_change(
        &self,
        device_id: String,
        channel: u32,
        parameter_type: crate::translation::types::UcNetParameterType,
        value: UcNetParameterValue,
    ) -> Vec<MidiMessageType> {
        let start_time = Instant::now();

        let param_id = ParameterId::new(device_id.clone(), channel, parameter_type);

        // Check if value has changed
        let mut shadow = self.shadow_state.write().await;
        if !shadow.has_changed(&param_id, &value) {
            debug!(
                "Filtered duplicate UCNet change: {} ch{} {:?}",
                device_id, channel, parameter_type
            );
            return Vec::new();
        }

        // Update shadow state
        shadow.update(param_id, value.clone());
        drop(shadow);

        // Perform reverse mapping (UCNet → MIDI)
        let mapper = self.mapper.read().await;
        let midi_messages = mapper.reverse_translate(&device_id, channel, parameter_type, value.clone());
        drop(mapper);

        // Measure latency
        let latency_ms = start_time.elapsed().as_secs_f64() * 1000.0;
        self.record_latency(latency_ms).await;

        // Emit events
        let _ = self.event_tx.send(SyncEvent::UcNetChanged {
            device_id: device_id.clone(),
            channel,
            parameter_type,
            value: value.clone(),
            timestamp: start_time,
        });

        let _ = self.event_tx.send(SyncEvent::ParameterSynced {
            source: SyncSource::UcNet,
            device_id: device_id.clone(),
            channel,
            parameter_type,
            latency_ms,
        });

        debug!(
            "UCNet→MIDI: {} ch{} {:?} = {:?} → {} MIDI messages ({:.2}ms)",
            device_id, channel, parameter_type, value, midi_messages.len(), latency_ms
        );

        midi_messages
    }

    /// Records a latency measurement
    async fn record_latency(&self, latency_ms: f64) {
        let mut samples = self.latency_samples.write().await;
        samples.push(latency_ms);

        // Keep only the most recent samples
        if samples.len() > self.max_latency_samples {
            samples.remove(0);
        }

        // Warn if latency exceeds target
        if latency_ms > 10.0 {
            warn!("Latency exceeded 10ms target: {:.2}ms", latency_ms);
        }
    }

    /// Gets latency statistics
    pub async fn get_latency_stats(&self) -> Option<LatencyStats> {
        let samples = self.latency_samples.read().await;

        if samples.is_empty() {
            return None;
        }

        let sum: f64 = samples.iter().sum();
        let avg = sum / samples.len() as f64;
        let min = samples.iter().copied().fold(f64::INFINITY, f64::min);
        let max = samples.iter().copied().fold(f64::NEG_INFINITY, f64::max);

        Some(LatencyStats {
            avg_ms: avg,
            min_ms: min,
            max_ms: max,
            sample_count: samples.len(),
        })
    }

    /// Clears latency statistics
    pub async fn clear_latency_stats(&self) {
        let mut samples = self.latency_samples.write().await;
        samples.clear();
    }

    /// Clears shadow state for a specific device
    pub async fn clear_device_state(&self, device_id: &str) {
        let mut shadow = self.shadow_state.write().await;
        shadow.clear_device(device_id);
    }

    /// Clears all shadow state
    pub async fn clear_all_state(&self) {
        let mut shadow = self.shadow_state.write().await;
        shadow.clear();
    }

    /// Starts a background task to clean up stale shadow state entries
    ///
    /// Returns a handle to the cleanup task that can be aborted.
    pub fn start_cleanup_task(
        shadow_state: Arc<RwLock<ShadowState>>,
        interval: Duration,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            loop {
                ticker.tick().await;
                let mut shadow = shadow_state.write().await;
                shadow.cleanup_stale();
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::translation::types::{TaperCurve, UcNetParameterType};

    #[tokio::test]
    async fn test_sync_engine_creation() {
        let (engine, _rx) = SyncEngine::default();
        assert_eq!(engine.get_mappings().await.len(), 0);
    }

    #[tokio::test]
    async fn test_add_and_get_mappings() {
        let (engine, _rx) = SyncEngine::default();

        let mapping = ParameterMapping::new_volume(
            0,
            7,
            "device-1".to_string(),
            1,
            TaperCurve::Linear,
        );

        engine.add_mapping(mapping).await;
        assert_eq!(engine.get_mappings().await.len(), 1);
    }

    #[tokio::test]
    async fn test_handle_midi_message() {
        let (engine, mut rx) = SyncEngine::default();

        let mapping = ParameterMapping::new_volume(
            0,
            7,
            "device-1".to_string(),
            1,
            TaperCurve::Linear,
        );
        engine.add_mapping(mapping).await;

        let message = MidiMessageType::ControlChange {
            channel: 0,
            controller: 7,
            value: 64,
        };

        let results = engine
            .handle_midi_message("midi-1".to_string(), message)
            .await;

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].device_id, "device-1");
        assert_eq!(results[0].channel, 1);

        // Check events
        let event = rx.recv().await.unwrap();
        match event {
            SyncEvent::ParameterSynced { source, latency_ms, .. } => {
                assert_eq!(source, SyncSource::Midi);
                assert!(latency_ms < 10.0); // Should be well under 10ms
            }
            _ => panic!("Expected ParameterSynced event"),
        }
    }

    #[tokio::test]
    async fn test_feedback_loop_prevention() {
        let (engine, _rx) = SyncEngine::default();

        let mapping = ParameterMapping::new_volume(
            0,
            7,
            "device-1".to_string(),
            1,
            TaperCurve::Linear,
        );
        engine.add_mapping(mapping).await;

        let message = MidiMessageType::ControlChange {
            channel: 0,
            controller: 7,
            value: 64,
        };

        // First message should produce result
        let results1 = engine
            .handle_midi_message("midi-1".to_string(), message.clone())
            .await;
        assert_eq!(results1.len(), 1);

        // Second identical message should be filtered
        let results2 = engine
            .handle_midi_message("midi-1".to_string(), message)
            .await;
        assert_eq!(results2.len(), 0);
    }

    #[tokio::test]
    async fn test_latency_stats() {
        let (engine, _rx) = SyncEngine::default();

        let mapping = ParameterMapping::new_volume(
            0,
            7,
            "device-1".to_string(),
            1,
            TaperCurve::Linear,
        );
        engine.add_mapping(mapping).await;

        // Generate some sync events
        for i in 0..10 {
            let message = MidiMessageType::ControlChange {
                channel: 0,
                controller: 7,
                value: i * 10,
            };
            engine
                .handle_midi_message("midi-1".to_string(), message)
                .await;
        }

        let stats = engine.get_latency_stats().await;
        assert!(stats.is_some());

        let stats = stats.unwrap();
        assert_eq!(stats.sample_count, 10);
        assert!(stats.avg_ms < 10.0); // Should be well under 10ms
        assert!(stats.min_ms >= 0.0);
        assert!(stats.max_ms >= stats.min_ms);
    }

    #[tokio::test]
    async fn test_clear_device_state() {
        let (engine, _rx) = SyncEngine::default();

        let mapping = ParameterMapping::new_volume(
            0,
            7,
            "device-1".to_string(),
            1,
            TaperCurve::Linear,
        );
        engine.add_mapping(mapping).await;

        let message = MidiMessageType::ControlChange {
            channel: 0,
            controller: 7,
            value: 64,
        };

        // Send message to create shadow state
        engine
            .handle_midi_message("midi-1".to_string(), message.clone())
            .await;

        // Clear device state
        engine.clear_device_state("device-1").await;

        // Same message should now produce result again
        let results = engine
            .handle_midi_message("midi-1".to_string(), message)
            .await;
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_handle_ucnet_change() {
        let (engine, mut rx) = SyncEngine::default();

        // Add a mapping for reverse translation
        let mapping = ParameterMapping::new_volume(
            0,
            7,
            "device-1".to_string(),
            1,
            TaperCurve::Linear,
        );
        engine.add_mapping(mapping).await;

        let results = engine
            .handle_ucnet_change(
                "device-1".to_string(),
                1,
                UcNetParameterType::Volume,
                UcNetParameterValue::Float(0.5),
            )
            .await;

        // Should return MIDI messages now
        assert_eq!(results.len(), 1);
        match results[0] {
            MidiMessageType::ControlChange { channel, controller, value } => {
                assert_eq!(channel, 0);
                assert_eq!(controller, 7);
                assert_eq!(value, 64); // 0.5 * 127 ≈ 64
            }
            _ => panic!("Expected ControlChange message"),
        }

        // Check events
        let event = rx.recv().await.unwrap();
        match event {
            SyncEvent::UcNetChanged { device_id, channel, .. } => {
                assert_eq!(device_id, "device-1");
                assert_eq!(channel, 1);
            }
            _ => panic!("Expected UcNetChanged event"),
        }
    }

    #[tokio::test]
    async fn test_ucnet_change_feedback_prevention() {
        let (engine, _rx) = SyncEngine::default();

        let mapping = ParameterMapping::new_volume(
            0,
            7,
            "device-1".to_string(),
            1,
            TaperCurve::Linear,
        );
        engine.add_mapping(mapping).await;

        // First change should produce MIDI messages
        let results1 = engine
            .handle_ucnet_change(
                "device-1".to_string(),
                1,
                UcNetParameterType::Volume,
                UcNetParameterValue::Float(0.5),
            )
            .await;
        assert_eq!(results1.len(), 1);

        // Second identical change should be filtered
        let results2 = engine
            .handle_ucnet_change(
                "device-1".to_string(),
                1,
                UcNetParameterType::Volume,
                UcNetParameterValue::Float(0.5),
            )
            .await;
        assert_eq!(results2.len(), 0);
    }
}
