//! Taper curve algorithms for parameter mapping
//!
//! Provides different response curves for translating MIDI values to UCNet parameters.
//! This is critical for achieving natural-feeling fader control.

use super::types::TaperCurve;

/// Applies a taper curve to a normalized input value (0.0 to 1.0)
///
/// # Arguments
/// * `input` - Normalized input value (0.0 to 1.0)
/// * `curve` - The taper curve to apply
///
/// # Returns
/// Normalized output value (0.0 to 1.0)
pub fn apply_taper(input: f32, curve: TaperCurve) -> f32 {
    // Clamp input to valid range
    let input = input.clamp(0.0, 1.0);
    
    match curve {
        TaperCurve::Linear => linear_taper(input),
        TaperCurve::Logarithmic => logarithmic_taper(input),
        TaperCurve::AudioTaper => audio_taper(input),
    }
}

/// Linear taper: 1:1 mapping
fn linear_taper(input: f32) -> f32 {
    input
}

/// Logarithmic taper: log(input + 1) / log(2)
///
/// Useful for frequency-like parameters where equal ratios should
/// produce equal perceptual changes.
fn logarithmic_taper(input: f32) -> f32 {
    if input <= 0.0 {
        0.0
    } else {
        // log(input + 1) / log(2) gives us a curve from 0 to 1
        (input + 1.0).log2() / 2.0_f32.log2()
    }
}

/// Audio taper: input^2.5
///
/// Approximates human hearing response for volume controls.
/// The exponent of 2.5 provides a good balance between linear and
/// logarithmic response for audio faders.
fn audio_taper(input: f32) -> f32 {
    if input <= 0.0 {
        0.0
    } else {
        input.powf(2.5)
    }
}

/// Converts a 7-bit MIDI value (0-127) to normalized float (0.0-1.0)
pub fn midi_7bit_to_normalized(value: u8) -> f32 {
    (value as f32) / 127.0
}

/// Converts a 14-bit MIDI value (0-16383) to normalized float (0.0-1.0)
pub fn midi_14bit_to_normalized(msb: u8, lsb: u8) -> f32 {
    let value = ((msb as u16) << 7) | (lsb as u16);
    (value as f32) / 16383.0
}

/// Converts normalized float (0.0-1.0) to 7-bit MIDI value (0-127)
pub fn normalized_to_midi_7bit(value: f32) -> u8 {
    let clamped = value.clamp(0.0, 1.0);
    (clamped * 127.0).round() as u8
}

/// Converts normalized float (0.0-1.0) to 14-bit MIDI value (MSB, LSB)
pub fn normalized_to_midi_14bit(value: f32) -> (u8, u8) {
    let clamped = value.clamp(0.0, 1.0);
    let midi_value = (clamped * 16383.0).round() as u16;
    let msb = ((midi_value >> 7) & 0x7F) as u8;
    let lsb = (midi_value & 0x7F) as u8;
    (msb, lsb)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_taper() {
        assert_eq!(apply_taper(0.0, TaperCurve::Linear), 0.0);
        assert_eq!(apply_taper(0.5, TaperCurve::Linear), 0.5);
        assert_eq!(apply_taper(1.0, TaperCurve::Linear), 1.0);
    }

    #[test]
    fn test_logarithmic_taper() {
        let result_0 = apply_taper(0.0, TaperCurve::Logarithmic);
        let result_half = apply_taper(0.5, TaperCurve::Logarithmic);
        let result_1 = apply_taper(1.0, TaperCurve::Logarithmic);
        
        assert_eq!(result_0, 0.0);
        assert!(result_half > 0.0 && result_half < 1.0);
        assert_eq!(result_1, 1.0);
        
        // Logarithmic curve: log(0.5 + 1) / log(2) = log(1.5) / log(2) ≈ 0.585
        // This is actually greater than linear (0.5) because log base 2 grows quickly
        assert!(result_half > 0.5);
        assert!((result_half - 0.585).abs() < 0.01);
    }

    #[test]
    fn test_audio_taper() {
        let result_0 = apply_taper(0.0, TaperCurve::AudioTaper);
        let result_half = apply_taper(0.5, TaperCurve::AudioTaper);
        let result_1 = apply_taper(1.0, TaperCurve::AudioTaper);
        
        assert_eq!(result_0, 0.0);
        assert!(result_half > 0.0 && result_half < 1.0);
        assert_eq!(result_1, 1.0);
        
        // Audio taper should be less than linear in the middle
        assert!(result_half < 0.5);
        // Expected value: 0.5^2.5 ≈ 0.177
        assert!((result_half - 0.177).abs() < 0.01);
    }

    #[test]
    fn test_taper_clamping() {
        // Test that values outside 0-1 are clamped
        assert_eq!(apply_taper(-0.5, TaperCurve::Linear), 0.0);
        assert_eq!(apply_taper(1.5, TaperCurve::Linear), 1.0);
    }

    #[test]
    fn test_midi_7bit_conversion() {
        assert_eq!(midi_7bit_to_normalized(0), 0.0);
        assert_eq!(midi_7bit_to_normalized(127), 1.0);
        assert!((midi_7bit_to_normalized(64) - 0.504).abs() < 0.01);
    }

    #[test]
    fn test_midi_14bit_conversion() {
        assert_eq!(midi_14bit_to_normalized(0, 0), 0.0);
        assert_eq!(midi_14bit_to_normalized(127, 127), 1.0);
        
        // Test MSB/LSB combination
        let mid_value = midi_14bit_to_normalized(64, 0);
        assert!((mid_value - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_normalized_to_midi_7bit() {
        assert_eq!(normalized_to_midi_7bit(0.0), 0);
        assert_eq!(normalized_to_midi_7bit(1.0), 127);
        assert_eq!(normalized_to_midi_7bit(0.5), 64);
        
        // Test clamping
        assert_eq!(normalized_to_midi_7bit(-0.5), 0);
        assert_eq!(normalized_to_midi_7bit(1.5), 127);
    }

    #[test]
    fn test_normalized_to_midi_14bit() {
        assert_eq!(normalized_to_midi_14bit(0.0), (0, 0));
        assert_eq!(normalized_to_midi_14bit(1.0), (127, 127));
        
        let (msb, lsb) = normalized_to_midi_14bit(0.5);
        assert_eq!(msb, 64);
        assert_eq!(lsb, 0);
    }

    #[test]
    fn test_round_trip_7bit() {
        // Test that converting back and forth preserves values
        for value in 0..=127 {
            let normalized = midi_7bit_to_normalized(value);
            let back = normalized_to_midi_7bit(normalized);
            assert_eq!(back, value);
        }
    }

    #[test]
    fn test_round_trip_14bit() {
        // Test a few key values
        let test_values = vec![(0, 0), (64, 0), (127, 127), (100, 50)];
        
        for (msb, lsb) in test_values {
            let normalized = midi_14bit_to_normalized(msb, lsb);
            let (back_msb, back_lsb) = normalized_to_midi_14bit(normalized);
            // Allow for small rounding errors
            assert!((back_msb as i16 - msb as i16).abs() <= 1);
            assert!((back_lsb as i16 - lsb as i16).abs() <= 1);
        }
    }
}
