use super::Processor;
use crate::audio::delay::DelayLine;

const BATCH_SIZE: usize = 32;
const MIN_DELAY: f32 = 0.001;
const MAX_DELAY: f32 = 5.0;

pub struct Delay {
    /// The left and right delay lines.
    delay_lines: [DelayLine; 2],
    /// The sample rate in `Hz`.
    sample_rate: f32,
    /// The target delay value in seconds.
    delay: f32,
    /// Feedback between `0.0` and `1.0`.
    feedback: f32,
}

impl Delay {
    pub fn new() -> Self {
        Self {
            delay_lines: [
                DelayLine::new(MAX_DELAY, BATCH_SIZE),
                DelayLine::new(MAX_DELAY, BATCH_SIZE),
            ],
            sample_rate: 0.0,
            delay: 0.001,
            feedback: 0.5,
        }
    }

    pub fn set_delay(&mut self, delay: f32) {
        self.delay = delay.clamp(MIN_DELAY, MAX_DELAY);
    }

    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.clamp(0.0, 2.0);
    }
}

impl Processor for Delay {
    fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate as f32;
        for line in self.delay_lines.iter_mut() {
            line.set_sample_rate(sample_rate);
            line.seek(self.delay);
        }
    }

    fn process(&mut self, data: super::ProcessorData) {
        let len = data.audio_in[0].len();

        for (idx, line) in self.delay_lines.iter_mut().enumerate() {
            line.set_target_delay(self.delay);

            // Input and output buffers
            let audio_in = data.audio_in.get(idx);
            let mut audio_out = data.audio_out.get_mut(idx);

            let mut i = 0;
            let mut buffer = [0.0f32; BATCH_SIZE];

            while i < len {
                // Determine number of samples to process
                let j = (i + BATCH_SIZE).min(len);
                let buffer = &mut buffer[..(j - i)];

                // Generate output from ring buffer
                line.read(buffer);

                // Write output to output buffer
                if let Some(audio_out) = audio_out.as_mut() {
                    audio_out[i..j].copy_from_slice(buffer);
                }

                // Combine input and feedback signals
                for sample in buffer.iter_mut() {
                    *sample *= self.feedback;
                }
                if let Some(audio_in) = audio_in {
                    for s in i..j {
                        buffer[s - i] += audio_in[s];
                    }
                }

                // Write input and feedback to ring buffer
                line.write(buffer);

                // Advance
                i = j;
            }
        }
    }
}
