use self::voice::Voice;
pub use self::voice::VoiceOpts;
use crate::{
    audio::buffer::StereoBufferMut,
    midi::{MidiEvent, TimedMidiEvent},
    processor::{Processor, ProcessorData, ProcessorDescription},
};

mod envelope;
pub mod oscillators;
mod voice;

pub struct Synth {
    pitch_bend_cents: usize,
    voices: Vec<Voice>,
    counter: usize,
}

#[derive(Copy, Clone)]
pub struct SynthOpts {
    pub num_voices: u8,
    pub voice_opts: VoiceOpts,
}

impl Synth {
    pub fn new(opts: SynthOpts) -> Self {
        let voice = Voice::new(opts.voice_opts);
        Self {
            pitch_bend_cents: 100,
            voices: std::iter::repeat(voice)
                .take(opts.num_voices as usize)
                .collect(),
            counter: 0,
        }
    }
}

impl Synth {
    fn process(&mut self, midi_in: &[TimedMidiEvent], mut audio_out: StereoBufferMut) {
        for TimedMidiEvent { event, .. } in midi_in {
            match event {
                MidiEvent::NoteOn { note, velocity, .. } => {
                    let voice = self
                        .voices
                        .iter_mut()
                        .min_by_key(|v| v.priority(*note))
                        .unwrap();
                    voice.trigger(*note, *velocity, self.counter);
                    self.counter += 1;
                }
                MidiEvent::NoteOff { note, .. } => {
                    if let Some(voice) = self.voices.iter_mut().find(|v| v.note() == Some(*note)) {
                        voice.release(self.counter);
                        self.counter += 1;
                    }
                }
                MidiEvent::PitchBend { value, .. } => {
                    let bend = calc_pitch_bend(*value, self.pitch_bend_cents);
                    for voice in &mut self.voices {
                        voice.set_pitch_bend(bend);
                    }
                }
                _ => {}
            }
        }

        audio_out.clear();
        for voice in &mut self.voices {
            voice.process(audio_out.left, audio_out.right);
        }
    }
}

impl Processor for Synth {
    fn description(&self) -> ProcessorDescription {
        ProcessorDescription {
            min_audio_ins: 0,
            max_audio_ins: 0,
            num_audio_outs: 2,
        }
    }

    fn set_sample_rate(&mut self, sample_rate: u32) {
        for voice in &mut self.voices {
            voice.set_sample_rate(sample_rate);
        }
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
