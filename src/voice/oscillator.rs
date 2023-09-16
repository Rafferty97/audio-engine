use crate::{audio::buffer::StereoBufferMut, note::Note};
use std::f32::consts::PI;

use super::{envelope::AdsrEnvelope, Voice};
#[derive(Clone, Copy)]
pub struct SimpleOscillator {
    inv_sample_rate: f32,
    wave: Waveform,
    note: Note,
    velocity: f32,
    phase: f32,
    bend: f32,
    envelope: AdsrEnvelope,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Waveform {
    Sine,
    Triangle,
    Square,
    Sawtooth,
}

impl SimpleOscillator {
    pub fn new() -> Self {
        Self {
            inv_sample_rate: 0.0,
            wave: Waveform::Sine,
            velocity: 0.0,
            note: Note::middle_c(),
            phase: 0.0,
            bend: 1.0,
            envelope: AdsrEnvelope::new(),
        }
    }
}

impl Voice for SimpleOscillator {
    fn set_sample_rate(&mut self, sample_rate: u32) {
        self.inv_sample_rate = (sample_rate as f32).recip();
        self.envelope.set_sample_rate(sample_rate);
    }

    fn trigger(&mut self, note: Note, velocity: u8) {
        self.note = note;
        self.velocity = (velocity as f32) / 127.0;
        self.envelope.trigger();
    }

    fn release(&mut self) {
        self.envelope.release();
    }

    fn set_pitch_bend(&mut self, bend: f32) {
        self.bend = bend;
    }

    fn process(&mut self, audio_out: StereoBufferMut) -> bool {
        let StereoBufferMut { left, right } = audio_out;

        let wave = match self.wave {
            Waveform::Sine => sine,
            Waveform::Triangle => triangle,
            Waveform::Square => square,
            Waveform::Sawtooth => sawtooth,
        };

        let omega = self.bend * self.note.frequency() * self.inv_sample_rate;
        for (left, right) in left.iter_mut().zip(right.iter_mut()) {
            let sample = self.envelope.process() * self.velocity * (wave)(self.phase);
            *left += sample;
            *right += sample;
            self.phase += omega;
            if self.phase >= 1.0 {
                self.phase -= 1.0;
            }
        }

        self.envelope.active()
    }
}

fn sine(phase: f32) -> f32 {
    (2.0 * PI * phase).sin()
}

fn square(phase: f32) -> f32 {
    if phase > 0.5 {
        1.0
    } else {
        -1.0
    }
}

fn triangle(phase: f32) -> f32 {
    (4.0 * phase - 2.0).abs() + 1.0
}

fn sawtooth(phase: f32) -> f32 {
    2.0 * phase - 1.0
}
