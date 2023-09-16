/// Converts a relative gain in dB to the corresponding voltage ratio/scaling factor.
pub fn scale_from_gain(gain: f32) -> f32 {
    10.0_f32.powf(gain / 20.0)
}

/// Converts a MIDI note value to a frequency in Hz.
pub fn hz_from_note(note: u8) -> f32 {
    440.0 * 2.0f32.powf((note as f32 - 69.0) / 12.0)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::f32::EPSILON;

    #[test]
    fn test_scale_from_gain() {
        assert_eq!(scale_from_gain(0.0), 1.0);
        assert!((scale_from_gain(40.0) - 100.0).abs() < EPSILON);
        assert!((scale_from_gain(20.0) - 10.0).abs() < EPSILON);
        assert!((scale_from_gain(-20.0) - 0.1).abs() < EPSILON);
        assert!((scale_from_gain(-40.0) - 0.01).abs() < EPSILON);
    }

    #[test]
    fn test_hz_from_note() {
        assert_eq!(hz_from_note(69), 440.0);
        assert_eq!(hz_from_note(69 + 12), 880.0);
        assert_eq!(hz_from_note(69 - 12), 220.0);
    }
}
