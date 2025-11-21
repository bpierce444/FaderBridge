//! MIDI to UCNet translation engine
//!
//! This module handles parameter mapping and taper curves for translating
//! MIDI messages to UCNet device parameters.

pub mod mapper;
pub mod taper;
pub mod types;

// Re-export commonly used types
pub use mapper::{MappingResult, ParameterMapper};
pub use taper::apply_taper;
pub use types::{ParameterMapping, TaperCurve, UcNetParameterType, UcNetParameterValue};
