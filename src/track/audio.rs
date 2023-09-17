use super::{TrackDuration, TrackPosition};
use crate::audio::{buffer::StereoBufferMut, sample::AudioSample};
use std::sync::Arc;

/// An audio track which contains audio clips.
pub struct AudioTrack {
    /// The clips on the track, in chronological order.
    clips: Vec<AudioClip>,
    /// The sample rate.
    sample_rate: f32,
    /// The clip which is currently playing, or was most recently played.
    clip_idx: usize,
    /// The state of the currently playing clip, if there is one playing.
    play_position: Option<ClipPlayPosition>,
}

/// An audio clip.
pub struct AudioClip {
    /// Start of the audio clip on the timeline.
    start: TrackPosition,
    /// Duration of the audio clip on the timeline.
    duration: TrackDuration,
    /// The audio sample.
    sample: Arc<AudioSample>,
    /// The sample rate of the clip.
    sample_rate: f32,
    /// The offset into the audio sample to begin playback from.
    sample_offset: usize,
}

/// Represents the state of an audio clip which is being played.
struct ClipPlayPosition {
    /// The index of the clip being played.
    clip_idx: usize,
    /// Position of the playhead in units of 64th notes.
    track_pos: TrackPosition,
    /// Position of the next sample to be played from the clip.
    sample_idx: usize,
}

impl AudioTrack {
    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate as f32;
    }

    pub fn process(&mut self, end_pos: TrackPosition, audio_out: StereoBufferMut) {
        // todo
    }

    fn seek(&mut self, pos: TrackPosition) {
        let current_clip = self
            .clips
            .iter()
            .enumerate()
            .take_while(|(_, c)| c.start() <= pos)
            .last()
            .and_then(|(i, c)| (c.end() > pos).then_some((i, c)));
        self.play_position = current_clip.map(|(clip_idx, clip)| {
            let sample_idx = 0; // FIXME
            ClipPlayPosition {
                clip_idx,
                track_pos: pos,
                sample_idx,
            }
        });
    }
}

impl AudioClip {
    fn start(&self) -> TrackPosition {
        self.start
    }

    fn end(&self) -> TrackPosition {
        self.start + self.duration
    }

    // fn sample_idx_at(&self, pos: TrackPosition) -> usize {
    //     let rel_pos = pos - self.start;
    //     let rel_sample = self.sample_rate *
    //     self.sample_offset + rel_sample
    // }
}
