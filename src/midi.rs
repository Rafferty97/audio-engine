use crate::note::Note;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct TimedMidiEvent {
    pub time: u32,
    pub event: MidiEvent,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum MidiEvent {
    NoteOn {
        channel: u8,
        note: Note,
        velocity: u8,
    },
    NoteOff {
        channel: u8,
        note: Note,
        velocity: u8,
    },
    ControlChange {
        channel: u8,
        control: u8,
        value: u8,
    },
    PitchBend {
        channel: u8,
        value: u16,
    },
    Invalid,
}

impl MidiEvent {
    pub fn from_raw(data: &[u8]) -> Self {
        match *data {
            [a @ 0x80..=0x8f, note, velocity] => MidiEvent::NoteOff {
                channel: a & 0x0f,
                note: note.into(),
                velocity,
            },
            [a @ 0x90..=0x9f, note, velocity] => MidiEvent::NoteOn {
                channel: a & 0x0f,
                note: note.into(),
                velocity,
            },
            [a @ 0xb0..=0xbf, control, value] => MidiEvent::ControlChange {
                channel: a & 0x0f,
                control,
                value,
            },
            [a @ 0xe0..=0xef, lsb, msb] => MidiEvent::PitchBend {
                channel: a & 0x0f,
                value: lsb as u16 | ((msb as u16) << 7),
            },
            _ => MidiEvent::Invalid,
        }
    }

    pub fn is_invalid(&self) -> bool {
        matches!(self, MidiEvent::Invalid)
    }
}
