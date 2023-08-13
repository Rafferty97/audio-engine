use std::f32::consts::PI;

use crate::{midi::MidiEvent, note::Note};

pub struct Voice {
    sample_rate: f32,
    cycle: f32,
    note: Note,
    on: bool,
    a: f32,
}

impl Voice {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate: sample_rate as f32,
            cycle: 0.0,
            note: Note::middle_c(),
            on: false,
            a: 0.0,
        }
    }
}

impl Voice {
    pub fn process(&mut self, midi_in: &[(u32, MidiEvent)], audio_out: &mut [f32]) {
        for (_, event) in midi_in {
            match event {
                MidiEvent::NoteOn { note, .. } => {
                    println!("ON {}", *note);
                    self.note = *note;
                    self.on = true;
                }
                MidiEvent::NoteOff { note, .. } if *note == self.note => {
                    println!("OFF");
                    self.on = false;
                }
                _ => {}
            }
        }

        let omega = 2.0 * PI * self.note.frequency() / self.sample_rate;
        for sample in audio_out.iter_mut() {
            *sample = self.a * self.cycle.sin();
            self.cycle += omega;
            self.a = f32::clamp(self.a + if self.on { 0.001 } else { -0.001 }, 0.0, 1.0);
        }
    }
}
