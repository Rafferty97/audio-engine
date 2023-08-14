use crate::midi::MidiEvent;
pub use autopan::Autopan;
pub use chord::Chord;
pub use pipeline::Pipeline;
pub use saturator::Saturator;

mod autopan;
mod chord;
mod pipeline;
mod saturator;

pub struct ProcessorData<'a> {
    /// List of input MIDI events
    pub midi_in: &'a [(u32, MidiEvent)],
    /// List of output MIDI events
    pub midi_out: &'a mut Vec<(u32, MidiEvent)>,
    /// List of input audio blocks
    pub audio_in: &'a [&'a [f32]],
    /// List of output audio blocks
    pub audio_out: &'a mut [&'a mut [f32]],
}

pub trait Processor {
    fn set_sample_rate(&mut self, sample_rate: u32);
    fn process(&mut self, data: ProcessorData);
}
