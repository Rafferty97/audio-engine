use crate::ring::RingBuffer;

use super::Processor;

pub struct Delay {
    buffers: [RingBuffer; 2],
    scratch: [f32; 2048],
    sample_rate: f32,
    delay: f32,
}

impl Delay {
    pub fn new() -> Self {
        Self {
            buffers: [RingBuffer::new(96000), RingBuffer::new(96000)],
            scratch: [0.0; 2048],
            sample_rate: 0.0,
            delay: 0.0,
        }
    }

    pub fn set_delay(&mut self, delay: f32) {
        self.delay = delay;
    }
}

impl Processor for Delay {
    fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate as f32;
    }

    fn process(&mut self, data: super::ProcessorData) {
        let len = data.audio_in[0].len();
        let delay_samples = (self.delay * self.sample_rate) as usize;

        for (idx, buffer) in self.buffers.iter_mut().enumerate() {
            buffer.sync_read_to_write(delay_samples);

            // Input
            if let Some(audio_in) = data.audio_in.get(idx) {
                buffer.write(audio_in);
            } else {
                buffer.mutate(len, |_, _| 0.0);
            }

            // Output
            if let Some(audio_out) = data.audio_out.get_mut(idx) {
                buffer.read(audio_out);
            }

            // Feedback
            let scratch = &mut self.scratch[..len];
            buffer.seek_read(-(len as isize));
            buffer.read(scratch);
            buffer.seek_write(-(len as isize));
            buffer.mutate(len, |i, s| s + 0.8 * scratch[i]);
        }
    }
}
