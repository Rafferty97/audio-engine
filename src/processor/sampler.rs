use crate::audio::{
    buffer::{StereoBuffer, StereoBufferMut},
    clip::AudioClip,
    resample::{CubicInterpolator, Resampler},
};

use super::Processor;

pub struct Sampler {
    /// Audio buffer containing the sample, partitioned in half
    /// with the left channel samples followed by the right channel samples.
    buffer: Vec<f32>,
    /// Current play position of the sample, in samples.
    read_idx: usize,
    /// The sample rate of the sample in the internal buffer
    sample_rate_in: f32,
    /// The sample rate of the audio output.
    sample_rate_out: f32,
    /// The samplers used to resample the left and right channels.
    samplers: [Resampler<CubicInterpolator>; 2],
    /// If `true`, the sampler does not repeat.
    one_hit: bool,
}

impl Sampler {
    pub fn new() -> Self {
        Self {
            // Initialise buffer with short length of silence
            buffer: vec![0.0; 1024],
            read_idx: 0,
            sample_rate_in: 0.0,
            sample_rate_out: 0.0,
            samplers: [Resampler::new(), Resampler::new()],
            one_hit: false, // FIXME
        }
    }

    pub fn set_sample(&mut self, clip: &AudioClip) {
        let data = clip.stereo_data();
        self.buffer.clear();
        self.buffer.extend(data.left);
        self.buffer.extend(data.right);
        self.sample_rate_in = clip.sample_rate() as f32;
        self.read_idx = 0;
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate_out = sample_rate as f32;
    }

    /// Returns the length of the internal sample in samples.
    fn length(&self) -> usize {
        self.buffer.len() / 2
    }

    pub fn process(&mut self, audio_out: StereoBufferMut) {
        let vout = audio_out;

        // Compute the resampling ratio
        let ratio = if self.sample_rate_in > 0.0 && self.sample_rate_out > 0.0 {
            self.sample_rate_in / self.sample_rate_out
        } else {
            1.0
        };

        // Fill the input buffers
        let input_size = self.samplers[0].next_input_size(vout.len(), ratio);
        let left = &mut [0.0; 4096][..input_size];
        let right = &mut [0.0; 4096][..input_size];
        self.fill_buffers(StereoBufferMut::new(left, right));

        // Perform the resampling directly into the output buffers
        let o1 = self.samplers[0].resample(left, vout.left, ratio);
        let o2 = self.samplers[1].resample(right, vout.right, ratio);
        debug_assert!(o1 == o2);
        if self.one_hit {
            self.read_idx = (self.read_idx + o1).min(self.length());
        } else {
            self.read_idx = (self.read_idx + o1) % self.length();
        }
    }

    /// Fills the provided buffer with raw audio from the internal sample,
    /// without advancing the read position into the sample.
    fn fill_buffers(&mut self, audio_out: StereoBufferMut) {
        let vin = split_buffer(&self.buffer);
        let mut vout = audio_out;

        let mut idx = self.read_idx;
        loop {
            let in_remain = vin.len() - idx;
            let out_remain = vout.len();
            if out_remain > in_remain {
                // Not enough samples left to fill the buffers, so need to repeat
                vout.slice_mut(..in_remain).copy(vin.slice(idx..));
                vout = vout.into_slice_mut(in_remain..);
                idx = 0;
                if self.one_hit {
                    vout.clear();
                    break;
                }
            } else {
                // Enough samples left to fill the buffers without repeating
                vout.copy(vin.slice(idx..(idx + out_remain)));
                break;
            }
        }
    }
}

impl Processor for Sampler {
    fn description(&self) -> super::ProcessorDescription {
        super::ProcessorDescription {
            min_audio_ins: 0,
            max_audio_ins: 0,
            num_audio_outs: 2,
        }
    }

    fn set_sample_rate(&mut self, sample_rate: u32) {
        self.set_sample_rate(sample_rate);
    }

    fn process(&mut self, data: super::ProcessorData) {
        let [left, right] = data.audio_out else {
            panic!("Expected at least two output audio buffers");
        };
        let audio_out = StereoBufferMut::new(left, right);

        self.process(audio_out);
    }
}

fn split_buffer(samples: &[f32]) -> StereoBuffer {
    let (left, right) = samples.split_at(samples.len() / 2);
    StereoBuffer::new(left, right)
}
