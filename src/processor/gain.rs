use super::Processor;

pub struct Gain {
    mul: f32,
}

impl Gain {
    pub fn new() -> Self {
        Self { mul: 1.0 }
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.mul = 10.0f32.powf(gain / 20.0);
    }
}

impl Processor for Gain {
    fn set_sample_rate(&mut self, sample_rate: u32) {}

    fn process(&mut self, data: super::ProcessorData) {
        let mul = self.mul;
        let buffers = data.audio_in.iter().zip(data.audio_out.iter_mut());
        for (buffer_in, buffer_out) in buffers {
            let samples = buffer_in.iter().zip(buffer_out.iter_mut());
            for (sample_in, sample_out) in samples {
                *sample_out = mul * *sample_in;
            }
        }
    }
}
