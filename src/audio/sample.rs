use super::buffer::{AudioBufferMut, MonoBuffer, StereoBuffer};
use crate::convert::uninterleave_stereo;
use std::io::Read;
use thiserror::Error;

/// A callback function for reporting progress of a long-running process.
type ProgressFn = Box<dyn FnMut(f64)>;

#[derive(Clone)]
pub struct AudioSample {
    channel_format: ChannelFormat,
    sample_rate: u32,
    length: usize,
    data: Box<[f32]>,
    peaks: Option<(f32, f32)>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChannelFormat {
    Mono,
    Stereo,
}

impl AudioSample {
    pub fn new_mono(sample_rate: u32, audio: MonoBuffer) -> Self {
        Self {
            channel_format: ChannelFormat::Mono,
            sample_rate,
            length: audio.len(),
            data: audio.channel().to_vec().into_boxed_slice(),
            peaks: None,
        }
    }

    pub fn new_stereo(sample_rate: u32, audio: StereoBuffer) -> Self {
        let mut data = Vec::with_capacity(2 * audio.len());
        data.extend_from_slice(audio.left);
        data.extend_from_slice(audio.right);

        Self {
            channel_format: ChannelFormat::Stereo,
            sample_rate,
            length: audio.len(),
            data: data.into_boxed_slice(),
            peaks: None,
        }
    }

    pub fn read_wav(reader: impl Read, progress: Option<ProgressFn>) -> Result<Self, ReadAudioClipError> {
        let mut wav = hound::WavReader::new(reader)?;

        // Extract information from the file header
        let spec = wav.spec();
        let length = wav.duration() as usize;
        let sample_rate = spec.sample_rate;
        let channels = spec.channels;
        let format = spec.sample_format;

        // Determine the maximum sample value, used to normalize the samples between -1.0 and 1.0
        let max_value = match spec.bits_per_sample {
            8 => 0x7f,
            16 => 0x7fff,
            24 => 0x7fffff,
            32 => 0x7fffffff,
            _ => return Err(ReadAudioClipError::UnexpectedError),
        };
        let scale = (max_value as f32).recip();

        // Read the interlaced samples into a buffer, normalized into `f32` values between -1.0 and 1.0
        let samples: Vec<f32> = match format {
            hound::SampleFormat::Int => wav
                .into_samples::<i32>()
                .map(|s| s.map(|s| s as f32 * scale))
                .collect::<Result<_, _>>(),
            hound::SampleFormat::Float => wav.into_samples::<f32>().collect(),
        }?;
        if samples.len() != channels as usize * length {
            return Err(ReadAudioClipError::UnexpectedError);
        }

        // De-interlace the samples
        let (channel_format, data) = match channels {
            1 => (ChannelFormat::Mono, samples.into_boxed_slice()),
            2 => {
                let mut data = vec![0.0; 2 * length];
                let (left, right) = data.split_at_mut(length);
                uninterleave_stereo(&samples, left, right);
                (ChannelFormat::Stereo, data.into_boxed_slice())
            }
            _ => return Err(ReadAudioClipError::BadFormat("Unsupported number of channels")),
        };

        // Construct the clip
        Ok(Self {
            channel_format,
            sample_rate,
            length,
            data,
            peaks: None,
        })
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn length(&self) -> usize {
        self.length
    }

    pub fn channels(&self) -> usize {
        match self.channel_format {
            ChannelFormat::Mono => 1,
            ChannelFormat::Stereo => 2,
        }
    }

    pub fn data(&self, channel: usize) -> &[f32] {
        self.data
            .chunks_exact(self.length)
            .nth(channel)
            .expect("Channel is out of range")
    }

    pub fn stereo_data(&self) -> StereoBuffer {
        match self.channel_format {
            // For mono clips, copy the mono buffer to both channels
            ChannelFormat::Mono => StereoBuffer::new(&self.data, &self.data),
            // For stereo clips, just pass the data as is
            ChannelFormat::Stereo => {
                let (left, right) = self.data.split_at(self.length);
                StereoBuffer::new(left, right)
            }
        }
    }

    pub fn trim(&self, start: usize, end: usize) -> AudioSample {
        // Clamp start and end indices and compute new length
        let start = start.clamp(0, self.length);
        let end = end.clamp(start, self.length);
        let length = end - start;

        // Trim the data in each channel
        let mut data = Vec::with_capacity(self.channels() * length);
        for channel in 0..self.channels() {
            data.extend_from_slice(&self.data(channel)[start..end]);
        }
        let data = data.into_boxed_slice();

        Self { data, length, ..*self }
    }

    /// Calculates the extreme values (minimum and maximum) of the samples across all channels.
    pub fn analyze_peaks(&mut self) -> (f32, f32) {
        *self.peaks.get_or_insert_with(|| {
            self.data
                .iter()
                .fold((0.0, 0.0), |(min, max), &s| (min.min(s), max.max(s)))
        })
    }

    /// Normalizes the audio clip such that the most extreme sample reaches a value of -1.0 or 1.0.
    pub fn normalize(&mut self) {
        let (min, max) = self.analyze_peaks();
        let peak = f32::max(-min, max);

        let scale = peak.recip();
        if !scale.is_finite() {
            return;
        }

        self.data.scale(scale);
    }
}

#[derive(Error, Debug)]
pub enum ReadAudioClipError {
    #[error("IO error: {0}")]
    IoError(std::io::Error),
    #[error("Format error: {0}")]
    BadFormat(&'static str),
    #[error("Unexpected error")]
    UnexpectedError,
}

impl From<hound::Error> for ReadAudioClipError {
    fn from(err: hound::Error) -> Self {
        use hound::Error as A;
        use ReadAudioClipError as B;
        match err {
            A::IoError(inner) => B::IoError(inner),
            A::FormatError(inner) => B::BadFormat(inner),
            A::Unsupported => B::BadFormat("Unsupported format"),
            _ => B::UnexpectedError,
        }
    }
}
