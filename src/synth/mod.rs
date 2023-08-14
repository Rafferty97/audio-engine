use self::voice::Voice;
pub use self::voice::VoiceOpts;
use crate::{
    convert::leftright_to_mono,
    midi::MidiEvent,
    processor::{Processor, ProcessorData},
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

impl Synth {}

impl Processor for Synth {
    fn set_sample_rate(&mut self, sample_rate: u32) {
        for voice in &mut self.voices {
            voice.set_sample_rate(sample_rate);
        }
    }

    fn process(&mut self, data: ProcessorData) {
        for (_, event) in data.midi_in {
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

        adapt_mono(data.audio_out, |left, right| {
            left.fill(0.0);
            right.fill(0.0);
            for voice in &mut self.voices {
                voice.process(left, right);
            }
        })
    }
}

/// Allows outputting stereo audio to either a stereo pair of channels, or a single mono buffer.
fn adapt_mono(buffers: &mut [&mut [f32]], f: impl FnOnce(&mut [f32], &mut [f32])) {
    match buffers {
        [] => {}
        [mono] => {
            let mut buffer = vec![0.0; 2 * mono.len()]; // FIXME: Don't init
            let (left, right) = buffer.split_at_mut(mono.len());
            f(left, right);
            leftright_to_mono(left, right, mono);
        }
        [left, right, ..] => {
            f(left, right);
        }
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
