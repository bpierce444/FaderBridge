//! Shadow state management for preventing feedback loops
//!
//! The shadow state tracks the last known value of each parameter to prevent
//! infinite feedback loops (A→B→A→B...). When a parameter change is received,
//! we check if it matches the shadow state before propagating it.

use crate::translation::types::{UcNetParameterType, UcNetParameterValue};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Unique identifier for a parameter (device + channel + type)
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct ParameterId {
    /// UCNet device ID
    pub device_id: String,
    /// Channel number
    pub channel: u32,
    /// Parameter type
    pub parameter_type: UcNetParameterType,
}

impl ParameterId {
    /// Creates a new parameter ID
    pub fn new(device_id: String, channel: u32, parameter_type: UcNetParameterType) -> Self {
        Self {
            device_id,
            channel,
            parameter_type,
        }
    }
}

/// Shadow state entry with timestamp
#[derive(Debug, Clone)]
struct ShadowEntry {
    /// Last known value
    value: UcNetParameterValue,
    /// Timestamp of last update
    timestamp: Instant,
}

/// Manages shadow state for all parameters
pub struct ShadowState {
    /// Map of parameter ID to shadow entry
    state: HashMap<ParameterId, ShadowEntry>,
    /// Tolerance for float comparison (to handle rounding errors)
    float_tolerance: f32,
    /// Maximum age for shadow state entries (prevents stale data)
    max_age: Duration,
}

impl ShadowState {
    /// Creates a new shadow state manager
    ///
    /// # Arguments
    /// * `float_tolerance` - Tolerance for comparing float values (default: 0.001)
    /// * `max_age` - Maximum age for shadow state entries (default: 5 seconds)
    pub fn new(float_tolerance: f32, max_age: Duration) -> Self {
        Self {
            state: HashMap::new(),
            float_tolerance,
            max_age,
        }
    }

    /// Creates a shadow state manager with default settings
    pub fn default() -> Self {
        Self::new(0.001, Duration::from_secs(5))
    }

    /// Updates the shadow state for a parameter
    ///
    /// # Arguments
    /// * `id` - Parameter identifier
    /// * `value` - New parameter value
    pub fn update(&mut self, id: ParameterId, value: UcNetParameterValue) {
        self.state.insert(
            id,
            ShadowEntry {
                value,
                timestamp: Instant::now(),
            },
        );
    }

    /// Checks if a parameter value has changed compared to shadow state
    ///
    /// Returns `true` if the value is different or if no shadow state exists.
    /// Returns `false` if the value matches the shadow state (potential feedback loop).
    ///
    /// # Arguments
    /// * `id` - Parameter identifier
    /// * `value` - New parameter value to check
    pub fn has_changed(&self, id: &ParameterId, value: &UcNetParameterValue) -> bool {
        match self.state.get(id) {
            Some(entry) => {
                // Check if entry is too old
                if entry.timestamp.elapsed() > self.max_age {
                    return true; // Treat as changed if stale
                }

                // Compare values based on type
                !self.values_equal(&entry.value, value)
            }
            None => true, // No shadow state, treat as changed
        }
    }

    /// Compares two parameter values for equality
    ///
    /// For float values, uses tolerance-based comparison to handle rounding errors.
    fn values_equal(&self, a: &UcNetParameterValue, b: &UcNetParameterValue) -> bool {
        match (a, b) {
            (UcNetParameterValue::Float(a_val), UcNetParameterValue::Float(b_val)) => {
                (a_val - b_val).abs() < self.float_tolerance
            }
            (UcNetParameterValue::Bool(a_val), UcNetParameterValue::Bool(b_val)) => a_val == b_val,
            _ => false, // Different types are not equal
        }
    }

    /// Clears all shadow state
    pub fn clear(&mut self) {
        self.state.clear();
    }

    /// Clears shadow state for a specific device
    ///
    /// # Arguments
    /// * `device_id` - Device ID to clear state for
    pub fn clear_device(&mut self, device_id: &str) {
        self.state.retain(|id, _| id.device_id != device_id);
    }

    /// Removes stale entries from shadow state
    ///
    /// This should be called periodically to prevent memory growth.
    pub fn cleanup_stale(&mut self) {
        let max_age = self.max_age;
        self.state.retain(|_, entry| entry.timestamp.elapsed() < max_age);
    }

    /// Gets the current shadow value for a parameter
    ///
    /// Returns `None` if no shadow state exists or if the entry is stale.
    pub fn get(&self, id: &ParameterId) -> Option<UcNetParameterValue> {
        self.state.get(id).and_then(|entry| {
            if entry.timestamp.elapsed() < self.max_age {
                Some(entry.value.clone())
            } else {
                None
            }
        })
    }

    /// Gets the number of entries in the shadow state
    pub fn len(&self) -> usize {
        self.state.len()
    }

    /// Checks if the shadow state is empty
    pub fn is_empty(&self) -> bool {
        self.state.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_shadow_state_update_and_get() {
        let mut shadow = ShadowState::default();
        let id = ParameterId::new("device-1".to_string(), 1, UcNetParameterType::Volume);
        let value = UcNetParameterValue::Float(0.5);

        shadow.update(id.clone(), value.clone());
        
        assert_eq!(shadow.get(&id), Some(value));
        assert_eq!(shadow.len(), 1);
    }

    #[test]
    fn test_has_changed_no_shadow() {
        let shadow = ShadowState::default();
        let id = ParameterId::new("device-1".to_string(), 1, UcNetParameterType::Volume);
        let value = UcNetParameterValue::Float(0.5);

        // No shadow state exists, should return true
        assert!(shadow.has_changed(&id, &value));
    }

    #[test]
    fn test_has_changed_same_value() {
        let mut shadow = ShadowState::default();
        let id = ParameterId::new("device-1".to_string(), 1, UcNetParameterType::Volume);
        let value = UcNetParameterValue::Float(0.5);

        shadow.update(id.clone(), value.clone());
        
        // Same value, should return false
        assert!(!shadow.has_changed(&id, &value));
    }

    #[test]
    fn test_has_changed_different_value() {
        let mut shadow = ShadowState::default();
        let id = ParameterId::new("device-1".to_string(), 1, UcNetParameterType::Volume);
        let value1 = UcNetParameterValue::Float(0.5);
        let value2 = UcNetParameterValue::Float(0.7);

        shadow.update(id.clone(), value1);
        
        // Different value, should return true
        assert!(shadow.has_changed(&id, &value2));
    }

    #[test]
    fn test_float_tolerance() {
        let mut shadow = ShadowState::new(0.01, Duration::from_secs(5));
        let id = ParameterId::new("device-1".to_string(), 1, UcNetParameterType::Volume);
        let value1 = UcNetParameterValue::Float(0.5);
        let value2 = UcNetParameterValue::Float(0.505); // Within tolerance
        let value3 = UcNetParameterValue::Float(0.52);  // Outside tolerance

        shadow.update(id.clone(), value1);
        
        // Within tolerance, should return false
        assert!(!shadow.has_changed(&id, &value2));
        
        // Outside tolerance, should return true
        assert!(shadow.has_changed(&id, &value3));
    }

    #[test]
    fn test_bool_comparison() {
        let mut shadow = ShadowState::default();
        let id = ParameterId::new("device-1".to_string(), 1, UcNetParameterType::Mute);
        let value1 = UcNetParameterValue::Bool(true);
        let value2 = UcNetParameterValue::Bool(false);

        shadow.update(id.clone(), value1.clone());
        
        // Same value
        assert!(!shadow.has_changed(&id, &value1));
        
        // Different value
        assert!(shadow.has_changed(&id, &value2));
    }

    #[test]
    fn test_clear() {
        let mut shadow = ShadowState::default();
        let id = ParameterId::new("device-1".to_string(), 1, UcNetParameterType::Volume);
        let value = UcNetParameterValue::Float(0.5);

        shadow.update(id.clone(), value);
        assert_eq!(shadow.len(), 1);
        
        shadow.clear();
        assert_eq!(shadow.len(), 0);
        assert!(shadow.is_empty());
    }

    #[test]
    fn test_clear_device() {
        let mut shadow = ShadowState::default();
        
        let id1 = ParameterId::new("device-1".to_string(), 1, UcNetParameterType::Volume);
        let id2 = ParameterId::new("device-2".to_string(), 1, UcNetParameterType::Volume);
        
        shadow.update(id1.clone(), UcNetParameterValue::Float(0.5));
        shadow.update(id2.clone(), UcNetParameterValue::Float(0.7));
        
        assert_eq!(shadow.len(), 2);
        
        shadow.clear_device("device-1");
        
        assert_eq!(shadow.len(), 1);
        assert!(shadow.get(&id1).is_none());
        assert!(shadow.get(&id2).is_some());
    }

    #[test]
    fn test_stale_entries() {
        let mut shadow = ShadowState::new(0.001, Duration::from_millis(100));
        let id = ParameterId::new("device-1".to_string(), 1, UcNetParameterType::Volume);
        let value = UcNetParameterValue::Float(0.5);

        shadow.update(id.clone(), value.clone());
        assert_eq!(shadow.get(&id), Some(value.clone()));
        
        // Wait for entry to become stale
        sleep(Duration::from_millis(150));
        
        // Should return None for stale entry
        assert!(shadow.get(&id).is_none());
        
        // has_changed should return true for stale entry
        assert!(shadow.has_changed(&id, &value));
    }

    #[test]
    fn test_cleanup_stale() {
        let mut shadow = ShadowState::new(0.001, Duration::from_millis(100));
        let id = ParameterId::new("device-1".to_string(), 1, UcNetParameterType::Volume);
        let value = UcNetParameterValue::Float(0.5);

        shadow.update(id.clone(), value);
        assert_eq!(shadow.len(), 1);
        
        // Wait for entry to become stale
        sleep(Duration::from_millis(150));
        
        shadow.cleanup_stale();
        assert_eq!(shadow.len(), 0);
    }

    #[test]
    fn test_multiple_parameters() {
        let mut shadow = ShadowState::default();
        
        let id1 = ParameterId::new("device-1".to_string(), 1, UcNetParameterType::Volume);
        let id2 = ParameterId::new("device-1".to_string(), 1, UcNetParameterType::Pan);
        let id3 = ParameterId::new("device-1".to_string(), 2, UcNetParameterType::Volume);
        
        shadow.update(id1.clone(), UcNetParameterValue::Float(0.5));
        shadow.update(id2.clone(), UcNetParameterValue::Float(0.0));
        shadow.update(id3.clone(), UcNetParameterValue::Float(0.7));
        
        assert_eq!(shadow.len(), 3);
        assert!(shadow.get(&id1).is_some());
        assert!(shadow.get(&id2).is_some());
        assert!(shadow.get(&id3).is_some());
    }
}
