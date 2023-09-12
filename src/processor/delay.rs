use super::Processor;
use crate::audio::{
    buffer::{AudioBufferMut, StereoBuffer, StereoBufferMut},
    delay_line::DelayLine,
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

    pub fn process(&mut self, audio_in: StereoBuffer, audio_out: StereoBufferMut) {
        let len = audio_in.len();
        assert!(audio_in.len() == audio_out.len());

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
            audio_out.left[i..j].copy(&*buffers[0]);
            audio_out.right[i..j].copy(&*buffers[1]);

            // Attenuate the output for feedback
            buffers[0].scale(self.feedback);
            buffers[1].scale(self.feedback);

            // Combine input and feedback signals, and write to ring buffers
            if self.ping_pong {
                // Write input only to right channel and swap feedback lines
                buffers[1].add_scaled(&audio_in.left[i..j], 0.5);
                buffers[1].add_scaled(&audio_in.right[i..j], 0.5);
                lines[0].write(buffers[1]);
                lines[1].write(buffers[0]);
            } else {
                // Write input to respective channels and don't swap feedback lines
                buffers[0].add(&audio_in.left[i..j]);
                buffers[1].add(&audio_in.right[i..j]);
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
        let [left, right, ..] = data.audio_in else {
            panic!("Expected at least two input audio buffers");
        };
        let audio_in = StereoBuffer::new(*left, *right);

        let [left, right, ..] = data.audio_out else {
            panic!("Expected at least two output audio buffers");
        };
        let audio_out = StereoBufferMut::new(*left, *right);

        self.process(audio_in, audio_out);
    }
}
