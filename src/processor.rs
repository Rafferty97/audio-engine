use crate::midi::MidiEvent;

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
    fn process(&mut self, data: ProcessorData);
}
