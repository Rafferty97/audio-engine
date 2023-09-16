use crate::{
    audio::buffer::StereoBufferMut,
    midi::{MidiEvent, TimedMidiEvent},
    note::Note,
    voice::Voice,
};

pub struct VoiceManager<V: Voice + Clone> {
    /// The maximum amount of pitch bend in cents
    max_pitch_bend: usize,
    /// The voices
    voices: Vec<VoiceHandle<V>>,
    /// Monotonic counter used to determine the least recently used voices
    counter: usize,
}

impl<V: Voice + Clone> VoiceManager<V> {
    pub fn new(num_voices: usize, voice: V) -> Self {
        let handle = VoiceHandle::new(voice);
        Self {
            max_pitch_bend: 100,
            voices: std::iter::repeat(handle).take(num_voices).collect(),
            counter: 0,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        for voice in &mut self.voices {
            voice.set_sample_rate(sample_rate);
        }
    }

    pub fn trigger(&mut self, note: Note, velocity: u8) {
        let voice = self.voices.iter_mut().min_by_key(|v| v.priority(note)).unwrap();
        voice.trigger(note, velocity, self.counter);
        self.counter += 1;
    }

    pub fn release(&mut self, note: Note) {
        if let Some(voice) = self.voices.iter_mut().find(|v| v.on_note() == Some(note)) {
            voice.release(self.counter);
            self.counter += 1;
        }
    }

    pub fn set_pitch_bend(&mut self, bend: f32) {
        for voice in &mut self.voices {
            voice.set_pitch_bend(bend);
        }
    }

    pub fn process(&mut self, mut audio_out: StereoBufferMut) {
        if audio_out.len() == 0 {
            return;
        }

        audio_out.clear();
        for voice in self.voices.iter_mut().filter(|v| v.active()) {
            voice.process(audio_out.as_mut());
        }
    }

    pub fn process_midi(&mut self, midi_in: &[TimedMidiEvent], audio_out: StereoBufferMut) {
        let mut vout = audio_out;

        for &TimedMidiEvent { time, event } in midi_in {
            // Process audio up to this event, and update the output buffer
            let time = time as usize;
            self.process(vout.slice_mut(..time));
            vout = vout.into_slice_mut(time..);

            // Process the MIDI event
            match event {
                MidiEvent::NoteOn { note, velocity, .. } => self.trigger(note, velocity),
                MidiEvent::NoteOff { note, .. } => self.release(note),
                MidiEvent::PitchBend { value, .. } => {
                    let bend = calc_pitch_bend(value, self.max_pitch_bend);
                    self.set_pitch_bend(bend);
                }
                _ => {}
            }
        }

        // Process the remainder of the output buffer
        self.process(vout);
    }
}

/// Converts a raw pitch bend into a scalar to be multiplied with frequency.
pub fn calc_pitch_bend(bend: u16, max_cents: usize) -> f32 {
    const MID_POINT: u16 = 8192; // No bend

    // Calculate how many cents the current bend represents
    let cents = ((bend as f32 - MID_POINT as f32) / MID_POINT as f32) * max_cents as f32;

    // Convert the bend in cents to a frequency scalar
    2f32.powf(cents / 1200f32)
}

#[derive(Clone)]
pub struct VoiceHandle<V: Voice> {
    voice: V,
    phase: VoicePhase,
    counter: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum VoicePhase {
    On(Note),
    Released(Note),
    Off,
}

impl<V: Voice> VoiceHandle<V> {
    pub fn new(voice: V) -> Self {
        Self {
            voice,
            phase: VoicePhase::Off,
            counter: 0,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.voice.set_sample_rate(sample_rate);
    }

    /// Returns `true` if the voice is sounding.
    pub fn active(&self) -> bool {
        self.phase != VoicePhase::Off
    }

    /// Gets the note that the voice is currently playing, if it is in the `On` phase.
    pub fn on_note(&self) -> Option<Note> {
        if let VoicePhase::On(note) = self.phase {
            Some(note)
        } else {
            None
        }
    }

    /// Gets the priority used for voice allocation, with the lowest priority being preferred.
    pub fn priority(&self, note: Note) -> usize {
        match self.phase {
            // Note has been retriggered
            VoicePhase::On(n) if n == note => 0,
            // Unused voice
            VoicePhase::Off => 1,
            // Released voice for the same note
            VoicePhase::Released(n) if n == note => 2,
            // Oldest released note
            VoicePhase::Released(_) => 3 + self.counter,
            // Oldest triggered note
            VoicePhase::On(_) => usize::MAX / 2 + self.counter,
        }
    }

    pub fn trigger(&mut self, note: Note, velocity: u8, counter: usize) {
        self.voice.trigger(note, velocity);
        self.phase = VoicePhase::On(note);
        self.counter = counter;
    }

    pub fn release(&mut self, counter: usize) {
        let note = match self.phase {
            VoicePhase::On(note) => note,
            VoicePhase::Released(note) => note,
            VoicePhase::Off => return,
        };

        self.voice.release();
        self.phase = VoicePhase::Released(note);
        self.counter = counter;
    }

    pub fn set_pitch_bend(&mut self, bend: f32) {
        self.voice.set_pitch_bend(bend);
    }

    /// Synthesises audio into the provided stereo buffer.
    /// A return value of `false` indicates that the voice is off and
    /// will not produce any more sound until it is re-triggered.
    pub fn process(&mut self, audio_out: StereoBufferMut) -> bool {
        if self.phase == VoicePhase::Off {
            return false;
        }

        let active = self.voice.process(audio_out);

        if !active {
            self.phase = VoicePhase::Off;
        }
        active
    }
}
