use crate::midi::MidiEvent;
pub use autopan::Autopan;
pub use chord::Chord;
pub use delay::Delay;
pub use gain::Gain;
pub use io::{AudioInput, AudioOutput};
pub use mixer::Mixer;
pub use pipeline::Pipeline;
pub use saturator::Saturator;

mod autopan;
mod chord;
mod delay;
mod gain;
mod io;
mod mixer;
mod pipeline;
mod saturator;
mod util;

pub struct ProcessorData<'a> {
    /// List of input MIDI events
    pub midi_in: &'a [(u32, MidiEvent)],
    /// List of output MIDI events
    pub midi_out: &'a mut Vec<(u32, MidiEvent)>,
    /// Number of samples in each audio block
    pub samples: usize,
    /// List of input audio blocks
    pub audio_in: &'a [&'a [f32]],
    /// List of output audio blocks
    pub audio_out: &'a mut [&'a mut [f32]],
}

#[derive(Copy, Clone)]
pub struct ProcessorDescription {
    pub min_audio_ins: usize,
    pub max_audio_ins: usize,
    pub num_audio_outs: usize,
}

pub trait Processor {
    fn description(&self) -> ProcessorDescription;
    fn set_sample_rate(&mut self, sample_rate: u32);
    fn process(&mut self, data: ProcessorData);
}
