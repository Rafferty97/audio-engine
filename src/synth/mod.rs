use self::voice::Voice;
use crate::{constants::DEFAULT_SAMPLE_RATE, midi::MidiEvent, note::Note, processor::Processor};
use std::f32::consts::PI;

mod voice;

pub struct Synth {
    sample_rate: f32,
    voice: Voice,
}

impl Synth {
    pub fn new() -> Self {
        Self {
            sample_rate: DEFAULT_SAMPLE_RATE as f32,
            voice: Voice::new(DEFAULT_SAMPLE_RATE),
        }
    }
}

impl Processor for Synth {
    fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate as f32;
        self.voice = Voice::new(DEFAULT_SAMPLE_RATE);
    }

    fn process(&mut self, data: crate::processor::ProcessorData) {
        let buffer = &mut data.audio_out[0];
        self.voice.process(data.midi_in, buffer);
    }
}
