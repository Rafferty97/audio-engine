use std::io::Read;
use thiserror::Error;

use crate::convert::uninterleave_stereo;

use super::buffer::{AudioBufferMut, StereoBuffer};

/// A callback function for reporting progress of a long-running process.
type ProgressFn = Box<dyn FnMut(f64)>;

#[derive(Clone)]
pub struct AudioClip {
    channel_format: ChannelFormat,
    sample_rate: u32,
    data: [Box<[f32]>; 2],
    peaks: Option<[(f32, f32); 2]>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChannelFormat {
    Mono,
    Stereo,
}

impl AudioClip {
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
            1 => (
                ChannelFormat::Mono,
                [samples.into_boxed_slice(), vec![].into_boxed_slice()],
            ),
            2 => {
                let mut left = vec![0.0; length];
                let mut right = vec![0.0; length];
                uninterleave_stereo(&samples, &mut left, &mut right);
                (
                    ChannelFormat::Stereo,
                    [left.into_boxed_slice(), right.into_boxed_slice()],
                )
            }
            _ => return Err(ReadAudioClipError::BadFormat("Unsupported number of channels")),
        };

        // Construct the clip
        Ok(Self {
            channel_format,
            sample_rate,
            data,
            peaks: None,
        })
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn stereo_data(&self) -> StereoBuffer {
        match self.channel_format {
            // For mono clips, copy the mono buffer to both channels
            ChannelFormat::Mono => StereoBuffer::new(&self.data[0], &self.data[0]),
            // For stereo clips, just pass the data as is
            ChannelFormat::Stereo => StereoBuffer::new(&self.data[0], &self.data[1]),
        }
    }

    pub fn trim(&self, start: usize, end: usize) -> AudioClip {
        let len = self.data[0].len();

        let start = start.clamp(0, len);
        let end = end.clamp(start, len);

        let data = [
            self.data[0][start..end].to_vec().into_boxed_slice(),
            if self.data[1].is_empty() {
                vec![].into_boxed_slice()
            } else {
                self.data[1][start..end].to_vec().into_boxed_slice()
            },
        ];

        Self { data, ..*self }
    }

    /// Calculates the extreme values (minimum and maximum) of the samples for each channel.
    pub fn analyze_peaks(&mut self) -> [(f32, f32); 2] {
        *self.peaks.get_or_insert_with(|| {
            [0, 1].map(|i| {
                (
                    self.data[i].iter().copied().reduce(f32::min).unwrap_or(0.0),
                    self.data[i].iter().copied().reduce(f32::max).unwrap_or(0.0),
                )
            })
        })
    }

    /// Normalizes the audio clip such that the most extreme sample reaches a value of -1.0 or 1.0.
    pub fn normalize(&mut self) {
        let peaks = self.analyze_peaks();
        let peak = peaks
            .iter()
            .map(|&(min, max)| f32::max(-min, max))
            .reduce(f32::max)
            .unwrap_or(0.0);

        let scale = peak.recip();
        if !scale.is_finite() {
            return;
        }

        for samples in &mut self.data {
            samples.scale(scale);
        }
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
