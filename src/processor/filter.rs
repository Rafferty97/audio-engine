use crate::audio::buffer::{AudioBuffer, AudioBufferMut, StereoBuffer, StereoBufferMut};
use std::{char::MAX, f32::consts::PI};

use super::Processor;

const MAX_COEFFS: usize = 8;

/// An infinite impulse response filter.
#[derive(Copy, Clone)]
pub struct IIRFilter {
    /// The coefficients, interlaced as [a0, b0, a1, b1, ...],
    /// and with the `a` coefficients inverted.
    coeffs: [f32; MAX_COEFFS],
    /// A buffer containing historical input and output samples from the filter,
    /// interlaced in the same manner as the filter coefficients, with outputs preceeding inputs.
    /// The most recent samples are at the start of the buffer.
    buffer: [f32; MAX_COEFFS],
}

impl IIRFilter {
    pub fn new_identity() -> Self {
        let mut coeffs = [0.0; MAX_COEFFS];
        coeffs[1] = 1.0;
        Self {
            coeffs,
            buffer: [0.0; MAX_COEFFS],
        }
    }

    pub fn new_lowpass(cutoff_hz: f32, sample_rate: f32) -> Self {
        let w = 2.0 * PI * cutoff_hz / sample_rate;
        let a = (w / 2.0).tan();

        let a0 = 1.0 + 2f32.sqrt() * a + a.powi(2);
        let mut coeffs = [
            0.0,
            1.0 / a0,
            (2.0 * a.powi(2) - 2.0) / a0,
            2.0 / a0,
            (-1.0 + 2f32.sqrt() * a - a.powi(2)) / a0,
            1.0 / a0,
            0.0,
            0.0,
        ];

        Self {
            coeffs,
            buffer: [0.0; MAX_COEFFS],
        }
    }

    pub fn process(&mut self, audio_in: &[f32], audio_out: &mut [f32]) {
        assert!(audio_in.len() == audio_out.len());
        audio_out.map(audio_in, |_, s| self.process_sample(s));
    }

    pub fn process_sample(&mut self, s_in: f32) -> f32 {
        // Shift the buffer and write the input sample.
        self.buffer.copy_within(..(MAX_COEFFS - 2), 2);
        self.buffer[1] = s_in;

        // Perform the convolution
        let s_out = self
            .coeffs
            .iter()
            .zip(self.buffer.iter())
            .map(|(c, s)| c * s)
            .sum();

        // Write the output sample and return it
        self.buffer[0] = s_out;
        s_out
    }
}

pub struct Filter {
    filters: [IIRFilter; 2],
    sample_rate: f32,
    cutoff: f32,
}

impl Filter {
    pub fn new() -> Self {
        Self {
            filters: [IIRFilter::new_identity(); 2],
            sample_rate: 0.0,
            cutoff: 0.0,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate as f32;
        self.calc_coefficients();
    }

    pub fn set_cutoff(&mut self, frequency: f32) {
        self.cutoff = frequency;
        self.calc_coefficients();
    }

    pub fn process(&mut self, audio_in: StereoBuffer, audio_out: StereoBufferMut) {
        self.filters[0].process(audio_in.left, audio_out.left);
        self.filters[1].process(audio_in.right, audio_out.right);
    }

    fn calc_coefficients(&mut self) {
        if self.sample_rate > 0.0 {
            self.filters = [IIRFilter::new_lowpass(self.cutoff, self.sample_rate); 2];
        }
    }
}

impl Processor for Filter {
    fn description(&self) -> super::ProcessorDescription {
        super::ProcessorDescription {
            min_audio_ins: 2,
            max_audio_ins: 2,
            num_audio_outs: 2,
        }
    }

    fn set_sample_rate(&mut self, sample_rate: u32) {
        self.set_sample_rate(sample_rate);
    }

    fn process(&mut self, data: super::ProcessorData) {
        let [left, right, ..] = data.audio_in else {
            panic!("Expected at least two input audio buffers");
        };
        let audio_in = StereoBuffer::new(left, right);

        let [left, right, ..] = data.audio_out else {
            panic!("Expected at least two output audio buffers");
        };
        let audio_out = StereoBufferMut::new(left, right);

        self.process(audio_in, audio_out);
    }
}
