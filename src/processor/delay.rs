use super::Processor;
use crate::audio::{
    delay_line::DelayLine,
    operations::{add_samples, add_scaled_samples, scale_samples},
};

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
    /// Whether "ping pong" delay is enabled.
    ping_pong: bool,
}

impl Delay {
    pub fn new() -> Self {
        Self {
            delay_lines: [DelayLine::new(MAX_DELAY), DelayLine::new(MAX_DELAY)],
            sample_rate: 0.0,
            delay: 0.001,
            feedback: 0.5,
            ping_pong: false,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate as f32;
        for line in self.delay_lines.iter_mut() {
            line.set_sample_rate(sample_rate);
            line.seek_seconds(self.delay);
        }
    }

    pub fn set_delay(&mut self, delay: f32) {
        self.delay = delay.clamp(MIN_DELAY, MAX_DELAY);
    }

    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.clamp(0.0, 2.0);
    }

    pub fn set_ping_pong(&mut self, ping_pong: bool) {
        self.ping_pong = ping_pong;
    }

    pub fn process(&mut self, audio_in: [&[f32]; 2], audio_out: [&mut [f32]; 2]) {
        let len = audio_in[0].len();
        debug_assert!(audio_in[0].len() == len);
        debug_assert!(audio_in[1].len() == len);
        debug_assert!(audio_out[0].len() == len);
        debug_assert!(audio_out[1].len() == len);

        let lines = &mut self.delay_lines;
        lines[0].set_target_delay(self.delay);
        lines[1].set_target_delay(self.delay);

        let mut i = 0;
        let mut buffer1 = [0.0f32; BATCH_SIZE];
        let mut buffer2 = [0.0f32; BATCH_SIZE];

        while i < len {
            // Determine number of samples to process
            let j = (i + BATCH_SIZE).min(len);
            let buffers = [&mut buffer1[..(j - i)], &mut buffer2[..(j - i)]];

            // Generate output from ring buffers
            lines[0].read(buffers[0]);
            lines[1].read(buffers[1]);

            // Write output to output buffers
            audio_out[0][i..j].copy_from_slice(buffers[0]);
            audio_out[1][i..j].copy_from_slice(buffers[1]);

            // Attenuate the output for feedback
            scale_samples(buffers[0], self.feedback);
            scale_samples(buffers[1], self.feedback);

            // Combine input and feedback signals, and write to ring buffers
            if self.ping_pong {
                // Write input only to right channel and swap feedback lines
                add_scaled_samples(buffers[1], &audio_in[0][i..j], 0.5);
                add_scaled_samples(buffers[1], &audio_in[1][i..j], 0.5);
                lines[0].write(buffers[1]);
                lines[1].write(buffers[0]);
            } else {
                // Write input to respective channels and don't swap feedback lines
                add_samples(buffers[0], &audio_in[0][i..j]);
                add_samples(buffers[1], &audio_in[1][i..j]);
                lines[0].write(buffers[0]);
                lines[1].write(buffers[1]);
            }

            // Advance
            i = j;
        }
    }
}

impl Processor for Delay {
    fn set_sample_rate(&mut self, sample_rate: u32) {
        self.set_sample_rate(sample_rate);
    }

    fn process(&mut self, data: super::ProcessorData) {
        let ([in1, in2], [out1, out2]) = (data.audio_in, data.audio_out) else {
            panic!("Incorrect number of audio buffers passed");
        };
        self.process([in1, in2], [out1, out2]);
    }
}
