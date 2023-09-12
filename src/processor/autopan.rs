use super::Processor;
use crate::audio::buffer::{AudioBufferMut, StereoBuffer, StereoBufferMut, StereoChannel};
use std::f32::consts::PI;

pub struct Autopan {
    inv_sample_rate: f32,
    frequency: f32,
    phase: f32,
    amount: f32,
}

impl Autopan {
    pub fn new(frequency: f32) -> Self {
        Self {
            inv_sample_rate: 0.0,
            frequency,
            phase: 0.0,
            amount: 1.0,
        }
    }

    pub fn set_amount(&mut self, amount: f32) {
        self.amount = amount;
    }

    pub fn process(&mut self, audio_in: StereoBuffer, mut audio_out: StereoBufferMut) {
        let phase = |n: usize| self.phase + self.frequency * self.inv_sample_rate * (n as f32);

        for channel in StereoChannel::both() {
            let buffer_in = audio_in.channel(channel);
            let buffer_out = audio_out.channel_mut(channel);
            let offset = match channel {
                StereoChannel::Left => 0.0,
                StereoChannel::Right => PI,
            };
            buffer_out.map(buffer_in, |i, sample| {
                let sin = (2.0 * PI * phase(i) + offset).sin();
                sample * (1.0 + self.amount * sin)
            })
        }

        self.phase = phase(audio_in.len());
        while self.phase > 1.0 {
            self.phase -= 1.0;
        }
    }
}

impl Processor for Autopan {
    fn set_sample_rate(&mut self, sample_rate: u32) {
        self.inv_sample_rate = (sample_rate as f32).recip();
    }

    fn process(&mut self, data: super::ProcessorData) {
        let [left, right, ..] = data.audio_in else {
            panic!("Expected at least two input audio buffers");
        };
        let audio_in = StereoBuffer::new(*left, *right);

        let [left, right, ..] = data.audio_out else {
            panic!("Expected at least two output audio buffers");
        };
        let audio_out = StereoBufferMut::new(*left, *right);

        self.process(audio_in, audio_out)
    }
}
