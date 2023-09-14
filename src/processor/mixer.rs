use super::Processor;
use crate::audio::buffer::{AudioBufferMut, StereoBufferMut};

const MAX_INPUTS: usize = 128;

pub struct Mixer {
    /// The gain factors for each input channel
    gains: [f32; MAX_INPUTS],
}

impl Mixer {
    pub fn new() -> Self {
        Self {
            gains: [1.0; MAX_INPUTS],
        }
    }

    pub fn set_gain(&mut self, input_idx: usize, gain: f32) {
        self.gains[input_idx] = 10.0f32.powf(gain / 20.0);
    }

    pub fn process(&mut self, audio_in: &[&[f32]], mut audio_out: StereoBufferMut) {
        audio_out.clear();
        for (idx, buffers) in audio_in.chunks_exact(2).enumerate() {
            audio_out.left.add_scaled(buffers[0], self.gains[idx]);
            audio_out.right.add_scaled(buffers[1], self.gains[idx]);
        }
    }
}

impl Processor for Mixer {
    fn description(&self) -> super::ProcessorDescription {
        super::ProcessorDescription {
            min_audio_ins: 0,
            max_audio_ins: 2 * MAX_INPUTS,
            num_audio_outs: 2,
        }
    }

    fn set_sample_rate(&mut self, _sample_rate: u32) {
        // Nothing to do
    }

    fn process(&mut self, data: super::ProcessorData) {
        if data.audio_in.len() > 2 * MAX_INPUTS {
            panic!("Too many input audio buffers");
        }

        let [left, right] = data.audio_out else {
            panic!("Incorrect number of audio buffers passed");
        };
        let audio_out = StereoBufferMut::new(left, right);

        self.process(data.audio_in, audio_out);
    }
}
