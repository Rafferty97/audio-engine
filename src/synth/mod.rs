use self::voice::VoiceManager;
use crate::{
    audio::buffer::StereoBufferMut,
    midi::{MidiEvent, TimedMidiEvent},
    processor::{Processor, ProcessorData, ProcessorDescription},
    voice::oscillator::SimpleOscillator,
};

mod voice;

pub struct SimpleSynth {
    voices: VoiceManager<SimpleOscillator>,
}

impl SimpleSynth {
    pub fn new() -> Self {
        Self {
            voices: VoiceManager::new(32, SimpleOscillator::new()),
        }
    }
}

impl SimpleSynth {
    fn process(&mut self, midi_in: &[TimedMidiEvent], audio_out: StereoBufferMut) {
        self.voices.process_midi(midi_in, audio_out)
    }
}

impl Processor for SimpleSynth {
    fn description(&self) -> ProcessorDescription {
        ProcessorDescription {
            min_audio_ins: 0,
            max_audio_ins: 0,
            num_audio_outs: 2,
        }
    }

    fn set_sample_rate(&mut self, sample_rate: u32) {
        self.voices.set_sample_rate(sample_rate)
    }

    fn process(&mut self, data: ProcessorData) {
        let [left, right] = data.audio_out else {
            panic!("Expected at least two output audio buffers");
        };
        let audio_out = StereoBufferMut::new(left, right);

        self.process(data.midi_in, audio_out);
    }
}

/// Converts a raw pitch bend into a scalar to be multiplied with frequency.
pub fn calc_pitch_bend(bend: u16, max_cents: usize) -> f32 {
    const MID_POINT: u16 = 8192; // No bend

    // Calculate how many cents the current bend represents
    let cents = ((bend as f32 - MID_POINT as f32) / MID_POINT as f32) * max_cents as f32;

    // Convert the bend in cents to a frequency scalar
    2f32.powf(cents / 1200f32)
}
