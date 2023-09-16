use std::{ops::Range, slice::SliceIndex};

pub trait AudioBuffer<'a>: Sized {
    fn samples(self) -> &'a [f32];
}

pub trait AudioBufferMut<'a>: AudioBuffer<'a> {
    fn samples_mut(self) -> &'a mut [f32];

    /// Fills the audio buffer with silence.
    fn clear(self) {
        self.samples_mut().fill(0.0);
    }

    /// Multiplies the samples by `sample`.
    fn scale(self, scale: f32) {
        for sample in self.samples_mut().iter_mut() {
            *sample *= scale;
        }
    }

    /// Clips the samples to within the range `min` to `max`.
    fn clip(self, min: f32, max: f32) {
        for sample in self.samples_mut().iter_mut() {
            *sample = sample.clamp(min, max);
        }
    }

    /// Copies the samples from `other` into this buffer.
    fn copy<'b>(self, other: impl AudioBuffer<'b>) {
        self.samples_mut().copy_from_slice(other.samples())
    }

    /// Copies the samples from `other` into this buffer, multiplied by `scale`.
    fn copy_scaled<'b>(self, other: impl AudioBuffer<'b>, scale: f32) {
        self.map(other, |_, s| scale * s)
    }

    /// Adds the samples from `other` to the samples in this buffer.
    fn add<'b>(self, other: impl AudioBuffer<'b>) {
        self.combine(other, |_, s_out, s_in| s_out + s_in)
    }

    /// Adds the samples from `other` to the samples in this buffer, multiplied by `scale`.
    fn add_scaled<'b>(self, other: impl AudioBuffer<'b>, scale: f32) {
        self.combine(other, |_, s_out, s_in| s_out + scale * s_in)
    }

    fn map<'b>(self, other: impl AudioBuffer<'b>, mut f: impl FnMut(usize, f32) -> f32) {
        let samples_in = other.samples();
        let samples_out = self.samples_mut();
        assert!(samples_in.len() == samples_out.len());
        for (idx, (s_out, s_in)) in samples_out.iter_mut().zip(samples_in.iter()).enumerate() {
            *s_out = (f)(idx, *s_in);
        }
    }

    fn combine<'b>(self, other: impl AudioBuffer<'b>, mut f: impl FnMut(usize, f32, f32) -> f32) {
        let samples_in = other.samples();
        let samples_out = self.samples_mut();
        assert!(samples_in.len() == samples_out.len());
        for (idx, (s_out, s_in)) in samples_out.iter_mut().zip(samples_in.iter()).enumerate() {
            *s_out = (f)(idx, *s_out, *s_in);
        }
    }
}

impl<'a> AudioBuffer<'a> for &'a [f32] {
    fn samples(self) -> &'a [f32] {
        self
    }
}

impl<'a> AudioBuffer<'a> for &'a mut [f32] {
    fn samples(self) -> &'a [f32] {
        self
    }
}

impl<'a> AudioBufferMut<'a> for &'a mut [f32] {
    fn samples_mut(self) -> &'a mut [f32] {
        self
    }
}

pub struct MonoBuffer<'a> {
    channel: &'a [f32],
}

impl<'a> MonoBuffer<'a> {
    pub fn new(channel: &'a [f32]) -> Self {
        Self { channel }
    }

    pub fn channel(&self) -> &[f32] {
        self.channel
    }
}

pub struct MonoBufferMut<'a> {
    channel: &'a mut [f32],
}

impl<'a> MonoBufferMut<'a> {
    pub fn new(channel: &'a mut [f32]) -> Self {
        Self { channel }
    }

    pub fn len(&self) -> usize {
        self.channel.len()
    }

    pub fn channel_mut(&mut self) -> &mut [f32] {
        self.channel
    }
}

#[derive(Clone, Copy)]
pub struct StereoBuffer<'a> {
    pub left: &'a [f32],
    pub right: &'a [f32],
}

impl<'a> StereoBuffer<'a> {
    pub fn new(left: &'a [f32], right: &'a [f32]) -> Self {
        assert!(left.len() == right.len());
        Self { left, right }
    }

    pub fn len(&self) -> usize {
        // Both channels must have the same length
        self.left.len()
    }

    pub fn channel(&self, channel: StereoChannel) -> &[f32] {
        match channel {
            StereoChannel::Left => self.left,
            StereoChannel::Right => self.right,
        }
    }

    pub fn slice(&self, range: impl SliceIndex<[f32], Output = [f32]> + Clone) -> StereoBuffer {
        StereoBuffer {
            left: &self.left[range.clone()],
            right: &self.right[range],
        }
    }
}

pub struct StereoBufferMut<'a> {
    pub left: &'a mut [f32],
    pub right: &'a mut [f32],
}

impl<'a> StereoBufferMut<'a> {
    pub fn new(left: &'a mut [f32], right: &'a mut [f32]) -> Self {
        assert!(left.len() == right.len());
        Self { left, right }
    }

    pub fn len(&self) -> usize {
        // Both channels must have the same length
        self.left.len()
    }

    pub fn channel(&self, channel: StereoChannel) -> &[f32] {
        match channel {
            StereoChannel::Left => self.left,
            StereoChannel::Right => self.right,
        }
    }

    pub fn channel_mut(&mut self, channel: StereoChannel) -> &mut [f32] {
        match channel {
            StereoChannel::Left => self.left,
            StereoChannel::Right => self.right,
        }
    }

    pub fn as_ref(&'a self) -> StereoBuffer<'a> {
        StereoBuffer::new(self.left, self.right)
    }

    pub fn as_mut(&mut self) -> StereoBufferMut {
        StereoBufferMut::new(self.left, self.right)
    }

    /// Fills all buffers with silence.
    pub fn clear(&mut self) {
        self.left.clear();
        self.right.clear();
    }

    pub fn copy(&mut self, other: StereoBuffer) {
        self.left.copy(other.left);
        self.right.copy(other.right);
    }

    pub fn slice(&self, range: impl SliceIndex<[f32], Output = [f32]> + Clone) -> StereoBuffer {
        StereoBuffer {
            left: &self.left[range.clone()],
            right: &self.right[range],
        }
    }

    pub fn slice_mut(&mut self, range: impl SliceIndex<[f32], Output = [f32]> + Clone) -> StereoBufferMut {
        StereoBufferMut {
            left: &mut self.left[range.clone()],
            right: &mut self.right[range],
        }
    }

    pub fn into_slice_mut(self, range: impl SliceIndex<[f32], Output = [f32]> + Clone) -> StereoBufferMut<'a> {
        StereoBufferMut {
            left: &mut self.left[range.clone()],
            right: &mut self.right[range],
        }
    }
}

#[derive(Clone, Copy)]
pub enum StereoChannel {
    Left = 0,
    Right = 1,
}

impl StereoChannel {
    pub const fn both() -> [StereoChannel; 2] {
        [StereoChannel::Left, StereoChannel::Right]
    }
}
