use super::Processor;
use crate::{midi::MidiEvent, note::Note};

pub struct Chord {
    channel: u8,
    chord: u64,
    notes: Vec<GeneratedNote>,
}

struct GeneratedNote {
    src: Note,
    dst: Note,
}

impl Chord {
    pub fn new() -> Self {
        Self {
            channel: 0,
            chord: 1,
            notes: vec![],
        }
    }

    pub fn set_channel(&mut self, channel: u8) {
        self.channel = channel;
    }

    pub fn set_chord(&mut self, chord: u64) {
        self.chord = chord;
    }

    fn chord_notes(&self) -> impl Iterator<Item = i8> {
        let chord = self.chord;
        (0..64).filter(move |i| (chord >> i) & 1 == 1)
    }

    fn get_mask(&self) -> u128 {
        to_mask(self.notes.iter().map(|n| n.dst))
    }

    pub fn process(&mut self, midi_in: &[(u32, MidiEvent)], midi_out: &mut Vec<(u32, MidiEvent)>) {
        for &(ts, event) in midi_in {
            match event {
                MidiEvent::NoteOn {
                    channel,
                    note,
                    velocity,
                } if channel == self.channel => {
                    let prev = self.get_mask();
                    self.notes.retain(|n| n.src != note);
                    for i in self.chord_notes() {
                        self.notes.push(GeneratedNote {
                            src: note,
                            dst: note.transpose(i),
                        });
                    }
                    let next = self.get_mask();
                    diff_masks(prev, next, |note, on| {
                        midi_out.push((
                            ts,
                            if on {
                                MidiEvent::NoteOn {
                                    channel,
                                    note,
                                    velocity,
                                }
                            } else {
                                MidiEvent::NoteOff {
                                    channel,
                                    note,
                                    velocity: 0,
                                }
                            },
                        ));
                    });
                }
                MidiEvent::NoteOff { channel, note, .. } if channel == self.channel => {
                    let prev = self.get_mask();
                    self.notes.retain(|n| n.src != note);
                    let next = self.get_mask();
                    diff_masks(prev, next, |note, on| {
                        midi_out.push((
                            ts,
                            if on {
                                unreachable!()
                            } else {
                                MidiEvent::NoteOff {
                                    channel,
                                    note,
                                    velocity: 0,
                                }
                            },
                        ));
                    });
                }
                _ => midi_out.push((ts, event)),
            }
        }
    }
}

fn to_mask(notes: impl IntoIterator<Item = Note>) -> u128 {
    let mut out = 0;
    for note in notes {
        out |= 1 << note.0;
    }
    out
}

fn diff_masks(prev: u128, next: u128, mut f: impl FnMut(Note, bool)) {
    for i in 0..128 {
        match ((prev >> i) & 1, (next >> i) & 1) {
            (0, 1) => f(Note(i), true),
            (1, 0) => f(Note(i), false),
            _ => {}
        }
    }
}

impl Processor for Chord {
    fn set_sample_rate(&mut self, _sample_rate: u32) {}

    fn process(&mut self, data: super::ProcessorData) {
        self.process(data.midi_in, data.midi_out)
    }
}
