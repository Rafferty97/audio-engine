use std::f32::consts::PI;

use super::Processor;

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
}

impl Processor for Autopan {
    fn set_sample_rate(&mut self, sample_rate: u32) {
        self.inv_sample_rate = (sample_rate as f32).recip();
    }

    fn process(&mut self, data: super::ProcessorData) {
        let phase = |n: usize| self.phase + self.frequency * self.inv_sample_rate * (n as f32);

        let buffers = data.audio_in.iter().zip(data.audio_out.iter_mut());
        for (i, (buffer_in, buffer_out)) in buffers.enumerate() {
            let samples = buffer_in.iter().zip(buffer_out.iter_mut());
            let offset = PI * i as f32;
            for (i, (sample_in, sample_out)) in samples.enumerate() {
                let sin = (2.0 * PI * phase(i) + offset).sin();
                *sample_out = *sample_in * (1.0 + self.amount * sin);
            }
        }

        self.phase = phase(data.audio_in[0].len());
        while self.phase > 1.0 {
            self.phase -= 1.0;
        }
    }
}
