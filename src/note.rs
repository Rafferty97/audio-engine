use std::sync::OnceLock;

use crate::util::hz_from_note;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Note(pub u8);

impl From<u8> for Note {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl std::fmt::Debug for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Note {
    pub fn middle_c() -> Self {
        Self(60)
    }

    pub fn name(&self) -> &'static str {
        note_name(self.0)
    }

    pub fn frequency(&self) -> f32 {
        hz_from_note(self.0)
    }

    pub fn transpose(&self, offset: i8) -> Self {
        Self(self.0.saturating_add_signed(offset))
    }
}

fn note_name(note: u8) -> &'static str {
    static NOTE_NAMES: OnceLock<[&str; 128]> = OnceLock::new();

    let names = NOTE_NAMES.get_or_init(|| {
        let octaves: [&[u8]; 11] = [b"-1", b"0", b"1", b"2", b"3", b"4", b"5", b"6", b"7", b"8", b"9"];
        let notes: [&[u8]; 12] = [
            b"C", b"C#", b"D", b"D#", b"E", b"F", b"F#", b"G", b"G#", b"A", b"A#", b"B",
        ];
        let buffer = octaves
            .iter()
            .flat_map(|octave| {
                notes.iter().flat_map(|note| {
                    let mut buffer = [b' '; 4];
                    buffer[..note.len()].copy_from_slice(note);
                    buffer[note.len()..note.len() + octave.len()].copy_from_slice(octave);
                    buffer
                })
            })
            .collect::<Vec<u8>>()
            .leak();
        core::array::from_fn(|i| std::str::from_utf8(&buffer[4 * i..4 * (i + 1)]).unwrap().trim())
    });

    names[note as usize]
}
