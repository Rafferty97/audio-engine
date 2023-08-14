use super::envelope::{AdsrEnvelope, AdsrPhase};
use crate::note::Note;

#[derive(Clone, Copy)]
pub struct Voice {
    inv_sample_rate: f32,
    wave: fn(f32) -> f32,
    note: Note,
    velocity: f32,
    phase: f32,
    envelope: AdsrEnvelope,
    counter: usize,
}

#[derive(Clone, Copy)]
pub struct VoiceOpts {
    /// The oscillator wave form.
    pub wave: fn(f32) -> f32,
    /// Attack time in seconds.
    pub attack: f32,
    /// Decay time in seconds.
    pub decay: f32,
    /// Sustain level between 0 and 1.
    pub sustain: f32,
    /// Release time in seconds.
    pub release: f32,
}

impl Voice {
    pub fn new(opts: VoiceOpts) -> Self {
        Self {
            inv_sample_rate: 0.0,
            wave: opts.wave,
            velocity: 0.0,
            note: Note::middle_c(),
            phase: 0.0,
            envelope: AdsrEnvelope::new(opts),
            counter: 0,
        }
    }
}

impl Voice {
    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.inv_sample_rate = (sample_rate as f32).recip();
        self.envelope.set_sample_rate(sample_rate);
    }

    /// Gets the note that the voice is currently playing, if it is in the `Active` phase.
    pub fn note(&self) -> Option<Note> {
        (self.envelope.phase() == AdsrPhase::Active).then_some(self.note)
    }

    /// Gets the priority used for voice allocation, with the lowest priority being preferred.
    pub fn priority(&self, note: Note) -> usize {
        match (self.note == note, self.envelope.phase()) {
            // Note has been retriggered
            (true, AdsrPhase::Active) => 0,
            // Unused voice
            (_, AdsrPhase::Inactive) => 1,
            // Released voice for the same note
            (true, AdsrPhase::Released) => 2,
            // Oldest released note
            (false, AdsrPhase::Released) => 3 + self.counter,
            // Oldest triggered note
            (false, AdsrPhase::Active) => usize::MAX / 2 + self.counter,
        }
    }

    pub fn trigger(&mut self, note: Note, velocity: u8, counter: usize) {
        self.note = note;
        self.velocity = (velocity as f32) / 127.0;
        self.envelope.trigger();
        self.counter = counter;
    }

    pub fn release(&mut self, counter: usize) {
        self.envelope.release();
        self.counter = counter;
    }

    pub fn process(&mut self, left: &mut [f32], right: &mut [f32]) {
        let omega = self.note.frequency() * self.inv_sample_rate;
        for (left, right) in left.iter_mut().zip(right.iter_mut()) {
            let sample = self.envelope.process() * self.velocity * (self.wave)(self.phase);
            *left += sample;
            *right += sample;
            self.phase += omega;
            if self.phase >= 1.0 {
                self.phase -= 1.0;
            }
        }
    }
}
