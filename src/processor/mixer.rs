use crate::audio::operations::add_scaled_samples;

use super::Processor;

const MAX_INPUTS: usize = 32;

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

    pub fn process(&mut self, audio_in: &[&[f32]], audio_out: &mut [f32]) {
        audio_out.fill(0.0);
        for (idx, buffer) in audio_in.iter().enumerate() {
            add_scaled_samples(audio_out, buffer, self.gains[idx]);
        }
    }
}

impl Processor for Mixer {
    fn set_sample_rate(&mut self, _sample_rate: u32) {
        // no-op
    }

    fn process(&mut self, data: super::ProcessorData) {
        if data.audio_in.len() > MAX_INPUTS {
            panic!("Incorrect number of audio buffers passed");
        }
        let [out] = data.audio_out else {
            panic!("Incorrect number of audio buffers passed");
        };
        self.process(data.audio_in, out);
    }
}
