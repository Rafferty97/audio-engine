use crate::note::Note;

#[derive(Clone, Copy)]
pub struct Voice {
    inv_sample_rate: f32,
    wave: fn(f32) -> f32,
    note: Option<Note>,
    velocity: f32,
    phase: f32,
    on: bool,
    a: f32,
    age: usize,
}

pub struct VoiceOpts {
    pub sample_rate: u32,
    pub wave: fn(f32) -> f32,
}

impl Voice {
    pub fn new(opts: VoiceOpts) -> Self {
        Self {
            inv_sample_rate: (opts.sample_rate as f32).recip(),
            wave: opts.wave,
            velocity: 0.0,
            note: None,
            phase: 0.0,
            on: false,
            a: 0.0,
            age: 0,
        }
    }
}

impl Voice {
    /// Gets the note that the voice is currently playing or last played.
    pub fn note(&self) -> Option<Note> {
        self.note
    }

    /// If the voice is not active, then it will produce silence until re-triggered.
    pub fn active(&self) -> bool {
        self.note.is_some()
    }

    pub fn priority(&self, note: Note) -> usize {
        match self.note {
            None => usize::MAX - 1,
            Some(n) if n == note => usize::MAX,
            Some(_) => {
                if self.on {
                    self.age
                } else {
                    self.age + usize::MAX / 2
                }
            }
        }
    }

    pub fn trigger(&mut self, note: Note, velocity: u8) {
        self.note = Some(note);
        self.velocity = 1.0; // (velocity as f32) / 127.0;
        self.on = true;
        self.age = 0;
    }

    pub fn release(&mut self) {
        self.on = false;
        self.age = 0;
    }

    pub fn process(&mut self, audio_out: &mut [f32]) {
        let Some(note) = self.note else {
            return;
        };

        let omega = note.frequency() * self.inv_sample_rate;
        for sample in audio_out.iter_mut() {
            *sample += self.a * self.velocity * (self.wave)(self.phase);
            self.phase += omega;
            if self.phase >= 1.0 {
                self.phase -= 1.0;
            }
            self.a = f32::clamp(self.a + if self.on { 0.001 } else { -0.001 }, 0.0, 1.0);
        }
        self.age += audio_out.len();

        if !self.on && self.a <= 0.0 {
            self.note = None;
        }
    }
}
