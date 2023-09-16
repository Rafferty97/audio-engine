use crate::{audio::buffer::StereoBufferMut, note::Note};

mod envelope;
pub mod oscillator;

/// A synthesiser or other instrument voice.
pub trait Voice {
    /// Sets the sample rate.
    fn set_sample_rate(&mut self, sample_rate: u32);

    /// Triggers a note to be played.
    fn trigger(&mut self, note: Note, velocity: u8);

    /// Releases the note.
    fn release(&mut self);

    /// Sets the pitch bend, where `bend` is a ratio to be multiplied with the original frequency.
    fn set_pitch_bend(&mut self, bend: f32);

    /// Synthesises audio into the provided stereo buffer.
    /// A return value of `false` indicates that the voice is off and
    /// will not produce any more sound until it is re-triggered.
    fn process(&mut self, audio_out: StereoBufferMut) -> bool;
}
