use std::f32::consts::PI;

use rand::Rng;

use crate::{constants::DEFAULT_SAMPLE_RATE, midi::MidiEvent, note::Note, processor::Processor};

pub struct Synth {
    sample_rate: f32,
    cycle: f32,
    note: Note,
    on: bool,
    a: f32,
}

impl Synth {
    pub fn new() -> Self {
        Self {
            sample_rate: DEFAULT_SAMPLE_RATE as f32,
            cycle: 0.0,
            note: Note::middle_c(),
            on: false,
            a: 0.0,
        }
    }
}

impl Processor for Synth {
    fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate as f32;
        self.on = false;
    }

    fn process(&mut self, data: crate::processor::ProcessorData) {
        for (_, event) in data.midi_in {
            match event {
                MidiEvent::NoteOn { note, .. } => {
                    self.note = *note;
                    self.on = true;
                }
                MidiEvent::NoteOff { note, .. } if *note == self.note => {
                    self.on = false;
                }
                _ => {}
            }
        }

        for buffer in data.audio_out.iter_mut() {
            let omega = 2.0 * PI * self.note.frequency() / self.sample_rate;
            for sample in buffer.iter_mut() {
                *sample = self.a * self.cycle.sin();
                self.cycle += omega;
                self.a = f32::clamp(self.a + if self.on { 0.001 } else { -0.001 }, 0.0, 1.0);
            }
        }
    }
}
