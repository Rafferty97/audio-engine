use super::{
    adapter::FixedOutputAdapter,
    resample::{CubicInterpolator, Resampler},
    ring::RingBuffer,
};

/// Fixed number of samples output by the resampler per cycle.
const OUTPUT_SIZE: usize = 32;

pub struct DelayLine {
    /// Inner ring buffer that stores the audio.
    ring: RingBuffer,
    /// Maximum delay value in seconds;
    /// used for calculating the ring buffer size.
    max_delay: f32,
    /// Sample rate in `Hz`.
    sample_rate: f32,
    /// Target delay value in samples.
    target_delay: usize,
    /// Current playback warp, in samples/second.
    /// A positive value causes faster than normal playback, and a negative value slower than normal playback.
    warp: f32,
    /// The resampler.
    resampler: Resampler<CubicInterpolator>,
    /// A small buffer for holding output.
    output_adapter: FixedOutputAdapter<OUTPUT_SIZE>,
}

impl DelayLine {
    /// Creates a new delay line with the given window size in seconds.
    /// The backing buffer isn't allocated until the sample rate has been set.
    pub fn new(max_delay: f32) -> Self {
        Self {
            ring: RingBuffer::new(0),
            max_delay,
            sample_rate: 0.0,
            target_delay: 0,
            warp: 0.0,
            resampler: Resampler::new(),
            output_adapter: FixedOutputAdapter::new(),
        }
    }

    /// Sets the sample rate. This clears the internal ring buffer.
    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate as f32;
        let size = (self.max_delay * sample_rate as f32) as usize;
        self.ring = RingBuffer::new(size);
    }

    /// Sets the delay of the read head to be the given number of seconds behind the write head.
    /// This takes effect instantaneously so may result in clicking/popping in the audio output.
    pub fn seek_seconds(&mut self, delay: f32) {
        self.seek_samples((delay * self.sample_rate) as usize);
    }

    /// Sets the delay of the read head to be the given number of samples behind the write head.
    /// This takes effect instantaneously so may result in clicking/popping in the audio output.
    pub fn seek_samples(&mut self, delay: usize) {
        self.target_delay = delay;
        let offset = self.resampler.reset();
        self.ring.seek(delay + offset);
    }

    /// Sets the delay of the read head to be the given number of seconds behind the write head,
    /// which will be smoothly transitioned to be speeding up or slowing down playback of the delayed signal.
    pub fn set_target_delay(&mut self, target_delay: f32) {
        self.target_delay = (target_delay * self.sample_rate) as usize;
    }

    /// Reads samples from the delay line.
    pub fn read(&mut self, samples: &mut [f32]) {
        let mut output = std::mem::take(&mut self.output_adapter);
        output.fill(samples, |buf| self.read_inner(buf));
        self.output_adapter = output;
    }

    fn read_inner(&mut self, samples: &mut [f32]) {
        debug_assert!(samples.len() == OUTPUT_SIZE);

        self.update_warp(OUTPUT_SIZE);

        // Set the resample ratio for this set of samples
        let ratio = (self.warp / self.sample_rate + 1.0).max(0.0);

        // Determine the number of samples to read
        let input_size = self.resampler.next_input_size(OUTPUT_SIZE, ratio);

        // If there are not enough samples available, return silence.
        if input_size > self.ring.delay() {
            samples.fill(0.0);
            return;
        }

        // Read samples from the ring buffer into the stack
        let read_buffer = &mut [0.0; 1024][..input_size];
        self.ring.read(read_buffer);

        // Perform the resampling directly into the output buffer
        self.resampler.resample(read_buffer, samples, ratio);
    }

    /// Write samples into the delay line.
    pub fn write(&mut self, samples: &[f32]) {
        self.ring.write(samples)
    }

    /// Updates the warp value using a critically damped oscillator
    /// to bring the actual delay towards the target delay.
    fn update_warp(&mut self, num_samples: usize) {
        // Controls how quickly the delay line repitches to the target delay value
        let omega = 8.0f32;

        // Compute the current delay error, in samples
        let error = self.target_delay as f32 - self.delay_samples();

        // If the error is very small, snap the delay and warp
        if error.abs() < 0.001 {
            self.seek_samples(self.target_delay);
            self.warp = 0.0;
            return;
        }

        // Compute the warp acceleration in samples/seconds^2
        let warp_acc = omega.powf(2.0) * error - 2.0 * omega * self.warp;

        // Apply the acceleration
        self.warp += warp_acc * (num_samples as f32 / self.sample_rate);
    }

    /// Gets the current delay in samples.
    pub fn delay_samples(&self) -> f32 {
        self.ring.delay() as f32 - self.resampler.position()
    }

    /// Gets the current delay in seconds.
    pub fn delay_seconds(&self) -> f32 {
        self.delay_samples() / self.sample_rate
    }
}
