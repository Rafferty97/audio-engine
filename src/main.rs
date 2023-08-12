use constants::DEFAULT_SAMPLE_RATE;
use midir::{Ignore, MidiInput};
use rand::Rng;
use rodio::source::SineWave;
use rodio::Source;
use std::io::{stdin, stdout, Write};
use std::time::Duration;

use crate::midi::MidiEvent;
use crate::processor::{Processor, ProcessorData};
use crate::synth::Synth;

mod constants;
mod midi;
mod note;
mod processor;
mod synth;

fn main() {
    // Get the default audio output device.
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();

    // Set up the MIDI input interface.
    let mut midi_in = MidiInput::new("MIDI input").unwrap();
    midi_in.ignore(Ignore::ActiveSense);

    // List available input ports.
    let in_ports = midi_in.ports();
    if in_ports.is_empty() {
        println!("No MIDI input ports available.");
        return;
    }

    // Prompt user to select a MIDI input port.
    println!("Available input ports:");
    for (i, p) in in_ports.iter().enumerate() {
        println!("{}: {}", i, midi_in.port_name(p).unwrap());
    }
    // print!("Please select input port: ");
    stdout().flush().unwrap();
    // let mut input = String::new();
    // stdin().read_line(&mut input).unwrap();
    // let input_port = input.trim().parse::<usize>().unwrap();
    let input_port = 0;
    let input_port = in_ports.into_iter().nth(input_port).unwrap();

    // Create a callback to handle incoming MIDI messages.
    let (midi_tx, midi_rx) = std::sync::mpsc::channel::<MidiEvent>();
    let callback = move |_, message: &[u8], _: &mut ()| {
        let event = MidiEvent::from_raw(message);
        if event.is_invalid() {
            return;
        }
        midi_tx.send(event).ok();
    };

    // Connect to the selected MIDI input port.
    let _connection = midi_in
        .connect(&input_port, "midi-read-connection", callback, ())
        .unwrap();

    // Create the synth
    let (audio_tx, audio_rx) = std::sync::mpsc::sync_channel(1);
    let source = BlockSource::new(audio_rx);
    std::thread::spawn(move || {
        let mut synth = Synth::new();
        'outer: loop {
            let mut events = vec![];
            loop {
                match midi_rx.try_recv() {
                    Ok(event) => events.push((0, event)),
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        break 'outer;
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => break,
                }
            }
            let mut samples = [0f32; 128];
            let mut midi_out = Vec::new();
            synth.process(ProcessorData {
                audio_in: &[],
                audio_out: &mut [&mut samples],
                midi_in: &events,
                midi_out: &mut midi_out,
            });
            if audio_tx.send(samples.into()).is_err() {
                break;
            }
        }
    });

    // let source = SineWave::new(440.0);

    // Play the sound on the default audio output.
    stream_handle.play_raw(source).unwrap();

    // Keep the program running.
    println!("Press Enter to exit...");
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
}

struct BlockSource {
    block: std::vec::IntoIter<f32>,
    rx: std::sync::mpsc::Receiver<Vec<f32>>,
    channels: u16,
    sample_rate: u32,
}

impl BlockSource {
    pub fn new(rx: std::sync::mpsc::Receiver<Vec<f32>>) -> Self {
        Self {
            block: Vec::new().into_iter(),
            rx,
            channels: 1,
            sample_rate: DEFAULT_SAMPLE_RATE,
        }
    }
}

impl Iterator for BlockSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        loop {
            if let Some(sample) = self.block.next() {
                return Some(sample);
            }
            self.block = self.rx.recv().ok()?.into_iter();
        }
    }
}

impl Source for BlockSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> u16 {
        self.channels
    }
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
