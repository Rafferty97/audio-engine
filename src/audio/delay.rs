use super::ring::RingBuffer;
use rubato::{FastFixedOut, Resampler};

pub struct DelayLine {
    /// Inner ring buffer that stores the audio.
    ring: RingBuffer,
    /// Maximum delay value in seconds;
    /// used for calculating the ring buffer size.
    max_delay: f32,
    /// Number of samples returned in each call to `read` or `read_resample`.
    output_size: usize,
    /// Sample rate in `Hz`.
    sample_rate: f32,
    /// Current warp in `samples/second`.
    warp: f32,
    /// Target delay value in samples.
    target_delay: usize,
    /// The resampler.
    sampler: FastFixedOut<f32>,
}

impl DelayLine {
    /// Creates a new delay line with the given window size in seconds.
    /// The backing buffer isn't allocated until the sample rate has been set.
    pub fn new(max_delay: f32, output_size: usize) -> Self {
        let sampler =
            FastFixedOut::new(1.0, 10.0, rubato::PolynomialDegree::Cubic, output_size, 1).unwrap();

        Self {
            ring: RingBuffer::new(0),
            max_delay,
            output_size,
            sample_rate: 0.0,
            warp: 0.0,
            target_delay: 0,
            sampler,
        }
    }

    /// Sets the sample rate. This clears the internal ring buffer.
    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate as f32;
        let size = (self.max_delay * sample_rate as f32) as usize;
        self.ring = RingBuffer::new(size);
    }

    /// Sets the delay of the read head to be the given number of sample behind the write head.
    pub fn seek(&mut self, delay: f32) {
        self.set_target_delay(delay);
        self.ring.seek(self.target_delay);
    }

    /// Sets the delay of the read head to be the given number of sample behind the write head.
    pub fn set_target_delay(&mut self, target_delay: f32) {
        let samples = (target_delay * self.sample_rate) as usize;
        self.target_delay = samples;
    }

    /// Reads samples from the ring buffer into `samples`, and advances the read position.
    pub fn read(&mut self, samples: &mut [f32]) {
        // Ensure output buffer is of the expected size
        assert_eq!(samples.len(), self.output_size);

        self.update_warp(self.output_size);

        // Set the resample ratio for this set of samples
        let ratio = ((self.warp / self.sample_rate) as f64 + 1.0).clamp(0.1, 10.0);
        self.sampler.set_resample_ratio(ratio, false).unwrap();

        // Determine the number of samples to read
        // If there are not enough samples available, return silence.
        let input_size = self.sampler.input_frames_next();
        if input_size > self.ring.delay() {
            samples.fill(0.0);
            return;
        }

        // Read samples from the ring buffer into the stack
        let read_buffer = &mut [0.0; 1024][..input_size];
        self.ring.read(read_buffer);

        // Perform the resampling directly into the output buffer
        self.sampler
            .process_into_buffer(&[read_buffer], &mut [samples], None)
            .unwrap();
    }

    /// Write samples from `samples` into the ring buffer, and advances the write position.
    pub fn write(&mut self, samples: &[f32]) {
        self.ring.write(samples)
    }

    /// Updates the warp value using a critically damped oscillator
    /// to bring the actual delay towards the target delay.
    fn update_warp(&mut self, samples: usize) {
        // Controls how quickly the delay line repitches to the target delay value
        let omega = 8.0f32;

        // Compute the current delay error, in samples
        let delay = self.ring.delay();
        let target_delay = self.target_delay;
        let error = target_delay as f32 - delay as f32;

        // Compute the warp acceleration in samples/seconds^2
        let warp_acc = omega.powf(2.0) * error - 2.0 * omega * self.warp;

        // Apply the acceleration
        self.warp += warp_acc * (samples as f32 / self.sample_rate);
    }
}
