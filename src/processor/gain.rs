use super::Processor;
use crate::{audio::buffer::AudioBufferMut, util::scale_from_gain};

pub struct Gain {
    scale: f32,
}

impl Default for Gain {
    fn default() -> Self {
        Self { scale: 1.0 }
    }
}

impl Gain {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.scale = scale_from_gain(gain);
    }

    pub fn process(&mut self, audio_in: &[&[f32]], audio_out: &mut [&mut [f32]]) {
        for (buf_in, buf_out) in audio_in.iter().zip(audio_out.iter_mut()) {
            buf_out.copy_scaled(*buf_in, self.scale);
        }
    }
}

impl Processor for Gain {
    fn description(&self) -> super::ProcessorDescription {
        super::ProcessorDescription {
            min_audio_ins: 0,
            max_audio_ins: 2,
            num_audio_outs: 2,
        }
    }

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
