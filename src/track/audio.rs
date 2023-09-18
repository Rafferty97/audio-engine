use crate::audio::{buffer::StereoBufferMut, sample::AudioSample};
use std::sync::Arc;

use super::timeline::Timeline;

/// An audio track which contains audio clips.
pub struct AudioTrack {
    /// The clips on the track, in chronological order.
    clips: Vec<AudioClip>,
    sample_rate: f32,
}

/// An audio clip.
pub struct AudioClip {
    /// Start of the audio clip on the timeline.
    start: f64,
    /// Duration of the audio clip on the timeline.
    duration: f64,
    /// The audio sample.
    sample: Arc<AudioSample>,
    /// The sample rate of the clip.
    sample_rate: f32,
    /// The offset into the audio sample to begin playback from.
    sample_offset: usize,
}

impl AudioTrack {
    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate as f32;
    }

    pub fn process(&self, timeline: &Timeline, mut audio_out: StereoBufferMut) {
        // TODO:
        //   - Get currently playing clip, if any (cache index)
        //   - If unwarped:
        //     - Calculate sample that clip started (cache)
        //     - Compute sample range to resample and emit
        //   - If warped:
        //     - Use beat and tempo to compute sample range to resample and emit

        let mut sample = timeline.curr_sample();
        for clips in self.clips.windows(2) {
            // Determine the number of samples to write from this clip
            let [clip, next_clip] = clips else {
                unreachable!("windows did not return 2 clips");
            };
            let next_trigger = timeline.time_to_sample(next_clip.start);
            if next_trigger <= sample {
                continue;
            }
            let len = (next_trigger - sample).min(audio_out.len());

            // Process the clip
            clip.process(timeline, sample, self.sample_rate, audio_out.slice_mut(..len));

            // Advance the playhead and audio buffer
            sample += len;
            audio_out = audio_out.into_slice_mut(len..);

            // Terminate when the buffer is full
            if audio_out.len() == 0 {
                break;
            }
        }
    }
}

impl AudioClip {
    fn start(&self) -> f64 {
        self.start
    }

    fn end(&self) -> f64 {
        self.start + self.duration
    }

    // fn sample_idx_at(&self, pos: TrackPosition) -> usize {
    //     let rel_pos = pos - self.start;
    //     let rel_sample = self.sample_rate *
    //     self.sample_offset + rel_sample
    // }

    fn process(&self, timeline: &Timeline, sample: usize, sample_rate: f32, mut audio_out: StereoBufferMut) {
        let start_sample = timeline.time_to_sample(self.start);
        let start_offset = sample - start_sample;
        let end_offset = start_offset + audio_out.len();
        let ratio = sample_rate / self.sample.sample_rate() as f32;
        let start = ratio * start_offset as f32;
        let end = ratio * end_offset as f32;

        // FIXME: Write samples start..end to audio_out
    }
}
