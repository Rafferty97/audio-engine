use super::Processor;

pub struct Saturator {
    curve: fn(f32) -> f32,
}

impl Saturator {
    pub fn new(curve: fn(f32) -> f32) -> Self {
        Self { curve }
    }
}

impl Processor for Saturator {
    fn set_sample_rate(&mut self, sample_rate: u32) {}

    fn process(&mut self, data: super::ProcessorData) {
        let buffers = data.audio_in.iter().zip(data.audio_out.iter_mut());
        for (buffer_in, buffer_out) in buffers {
            let samples = buffer_in.iter().zip(buffer_out.iter_mut());
            for (sample_in, sample_out) in samples {
                *sample_out = (self.curve)(*sample_in);
            }
        }
    }
}
