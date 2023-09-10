use std::marker::PhantomData;

pub trait Interpolator {
    /// Returns the number of samples needed on each side of the interpolated pair
    /// to perform the interpolation.
    fn window() -> usize;

    /// Performs the interpolation.
    fn interpolate(t: f32, samples: &[f32]) -> f32;
}

pub struct Resampler<I: Interpolator> {
    x1: f32,
    _interpolator: PhantomData<I>,
}

impl<I: Interpolator> Resampler<I> {
    /// Creates a new resampler.
    pub fn new() -> Self {
        Self {
            x1: I::window() as f32,
            _interpolator: PhantomData,
        }
    }

    /// Resets the resampler and returns the sample delay.
    pub fn reset(&mut self) -> usize {
        self.x1 = I::window() as f32;
        I::window()
    }

    /// Gets the position of the next sample to be interpolated, which might be fractional.
    pub fn position(&self) -> f32 {
        self.x1
    }

    /// Calculates the number of samples needed for the next call to `resample`.
    pub fn next_input_size(&self, output_samples: usize, ratio: f32) -> usize {
        let x2 = self.x1 + ratio * output_samples as f32;
        x2.floor() as usize + 2 + I::window()
    }

    /// Resamples the samples in `samples_in` into `samples_out`,
    /// returning the number of samples by which to advance the read window.
    ///
    /// # Parameters
    /// * `samples_in` - The input sample buffer.
    /// * `sampled_out` - The output sample buffer.
    /// * `ratio` - The ratio of input samples to output samples.
    pub fn resample(&mut self, samples_in: &[f32], samples_out: &mut [f32], ratio: f32) -> usize {
        // Fast path for when no actual resampling is occuring
        if self.x1.fract() == 0.0 && ratio == 1.0 {
            let x1 = self.x1 as usize;
            samples_out.copy_from_slice(&samples_in[x1..(x1 + samples_out.len())]);
            let offset = samples_out.len() + x1 - I::window();
            self.x1 = I::window() as f32;
            return offset;
        }

        let x1 = self.x1;
        let x2 = x1 + ratio * samples_out.len() as f32;

        // Ensure there are enough input samples and that `x1` and `x2` are within bounds
        let x_min = I::window() as f32;
        let x_max = (samples_in.len() - I::window() - 1) as f32;
        assert!(samples_in.len() >= 2 + I::window() + I::window());
        assert!(x1 >= x_min && x1 <= x_max);
        assert!(x2 >= x_min && x2 <= x_max);

        for (i, sample_out) in samples_out.iter_mut().enumerate() {
            let x = x1 + ratio * i as f32;
            let idx = x.floor() as usize - I::window();
            let frac = x.fract();
            *sample_out = I::interpolate(frac, &samples_in[idx..]);
        }

        let offset = x2.floor() - I::window() as f32;
        self.x1 = x2 - offset;
        offset as usize
    }
}

pub struct CubicInterpolator;

impl Interpolator for CubicInterpolator {
    #[inline]
    fn window() -> usize {
        1
    }

    #[inline]
    fn interpolate(t: f32, samples: &[f32]) -> f32 {
        let a0 = samples[1];
        let a1 =
            -(1.0 / 3.0) * samples[0] - (0.5) * samples[1] + samples[2] - (1.0 / 6.0) * samples[3];
        let a2 = (0.5) * (samples[0] + samples[2]) - samples[1];
        let a3 = (0.5) * (samples[1] - samples[2]) + (1.0 / 6.0) * (samples[3] - samples[0]);
        let x2 = t * t;
        let x3 = x2 * t;
        a0 + a1 * t + a2 * x2 + a3 * x3
    }
}
