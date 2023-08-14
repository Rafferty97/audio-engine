use self::voice::{Voice, VoiceOpts};
use crate::{midi::MidiEvent, processor::Processor};

mod oscillators;
mod voice;

pub struct Synth {
    voices: Vec<Voice>,
}

#[derive(Copy, Clone)]
pub struct SynthOpts {
    pub sample_rate: u32,
    pub num_voices: u8,
}

impl Synth {
    pub fn new(opts: SynthOpts) -> Self {
        let voice = Voice::new(VoiceOpts {
            sample_rate: opts.sample_rate,
            wave: self::oscillators::saw,
        });
        Self {
            voices: std::iter::repeat(voice)
                .take(opts.num_voices as usize)
                .collect(),
        }
    }
}

impl Synth {}

impl Processor for Synth {
    fn process(&mut self, data: crate::processor::ProcessorData) {
        for (_, event) in data.midi_in {
            match event {
                MidiEvent::NoteOn { note, velocity, .. } => {
                    let voice = self
                        .voices
                        .iter_mut()
                        .max_by_key(|v| v.priority(*note))
                        .unwrap();
                    voice.trigger(*note, *velocity)
                }
                MidiEvent::NoteOff { note, .. } => {
                    if let Some(voice) = self.voices.iter_mut().find(|v| v.note() == Some(*note)) {
                        voice.release()
                    }
                }
                _ => {}
            }
        }

        for voice in &mut self.voices {
            voice.process(data.audio_out[0]);
        }
    }
}
