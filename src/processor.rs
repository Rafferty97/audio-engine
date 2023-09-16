use crate::midi::TimedMidiEvent;
pub use autopan::Autopan;
pub use chord::Chord;
pub use delay::Delay;
pub use filter::Filter;
pub use gain::Gain;
pub use io::{AudioInput, AudioOutput, MidiInput};
pub use mixer::Mixer;
pub use pipeline::Pipeline;
pub use sampler::Sampler;
pub use saturator::Saturator;

mod autopan;
mod chord;
mod delay;
mod filter;
mod gain;
mod io;
mod mixer;
mod pipeline;
mod sampler;
mod saturator;

pub struct ProcessorData<'a> {
    /// List of input MIDI events
    pub midi_in: &'a [TimedMidiEvent],
    /// List of output MIDI events
    pub midi_out: &'a mut Vec<TimedMidiEvent>,
    /// Number of samples in each audio block
    pub samples: usize,
    /// List of input audio blocks
    pub audio_in: &'a [&'a [f32]],
    /// List of output audio blocks
    pub audio_out: &'a mut [&'a mut [f32]],
}

#[derive(Copy, Clone, Debug)]
pub struct ProcessorDescription {
    pub min_audio_ins: usize,
    pub max_audio_ins: usize,
    pub num_audio_outs: usize,
}

pub trait Processor: std::any::Any {
    /// Gets information about the processor.
    fn description(&self) -> ProcessorDescription;

    /// Provides the audio sample rate to the processor.
    /// This must be called before calling `process` or that method may panic.
    fn set_sample_rate(&mut self, sample_rate: u32) {}

    /// Sets the value of an automatable parameter.
    fn set_parameter(&mut self, param_id: usize, value: f32) {}

    /// Processes a batch of MIDI and audio data.
    fn process(&mut self, data: ProcessorData);
}
