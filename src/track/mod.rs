mod audio;

/// Represents a position on the timeline, in units of 64th notes.
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct TrackPosition(f64);

/// Represents a position on the timeline, in units of 64th notes.
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct TrackDuration(f64);

impl Eq for TrackPosition {}

// impl Ord for TrackPosition {
//     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//         self.0.partial_cmp(&other.0).unwrap()
//     }
// }

impl std::ops::Add<TrackDuration> for TrackPosition {
    type Output = TrackPosition;
    fn add(self, rhs: TrackDuration) -> Self::Output {
        TrackPosition(self.0 + rhs.0)
    }
}

impl std::ops::Sub<TrackPosition> for TrackPosition {
    type Output = TrackDuration;
    fn sub(self, rhs: TrackPosition) -> Self::Output {
        TrackDuration(self.0 - rhs.0)
    }
}
