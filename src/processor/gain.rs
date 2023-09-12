use super::Processor;
use crate::audio::buffer::AudioBufferMut;

pub struct Gain {
    scale: f32,
}

impl Gain {
    pub fn new() -> Self {
        Self { scale: 1.0 }
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.scale = 10.0f32.powf(gain / 20.0);
    }

    pub fn process(&mut self, audio_in: &[&[f32]], audio_out: &mut [&mut [f32]]) {
        for (buf_in, buf_out) in audio_in.iter().zip(audio_out.iter_mut()) {
            buf_out.copy_scaled(*buf_in, self.scale);
        }
    }
}

impl Processor for Gain {
    fn set_sample_rate(&mut self, sample_rate: u32) {
        // Nothing to do
    }

    fn process(&mut self, data: super::ProcessorData) {
        let mul = self.scale;
        let buffers = data.audio_in.iter().zip(data.audio_out.iter_mut());
        for (buffer_in, buffer_out) in buffers {
            let samples = buffer_in.iter().zip(buffer_out.iter_mut());
            for (sample_in, sample_out) in samples {
                *sample_out = mul * *sample_in;
            }
        }
    }
}
