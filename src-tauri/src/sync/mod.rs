//! Bidirectional synchronization engine
//!
//! This module handles state management, feedback loop prevention, and
//! orchestrates synchronization between MIDI controllers and UCNet devices.

pub mod engine;
pub mod shadow_state;

pub use engine::{LatencyStats, SyncEngine, SyncEvent, SyncSource};
pub use shadow_state::{ParameterId, ShadowState};
