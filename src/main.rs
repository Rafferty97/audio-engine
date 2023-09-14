use crate::midi::MidiEvent;
use crate::note::Note;
use crate::processor::AudioOutput;
use basedrop::Collector;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use engine::AudioEngine;
use midir::{Ignore, MidiInput};
use processor::AudioInput;
use std::time::Duration;

mod audio;
mod convert;
mod engine;
mod midi;
mod note;
mod processor;
mod synth;

fn main() {
    // Set up the MIDI input interface
    let mut midi_in = MidiInput::new("MIDI input").unwrap();
    midi_in.ignore(Ignore::ActiveSense);

    // Get or generate MIDI input
    let (midi_tx, midi_rx) = std::sync::mpsc::channel();
    let in_ports = midi_in.ports();
    let _connection;
    if !in_ports.is_empty() {
        // Create a callback to handle incoming MIDI messages
        let callback = move |_, message: &[u8], _: &mut ()| {
            let event = MidiEvent::from_raw(message);
            if event.is_invalid() {
                return;
            }
            midi_tx.send(event).ok();
        };

        // Connect to the selected MIDI input port
        _connection = midi_in
            .connect(&in_ports[0], "midi-read-connection", callback, ())
            .unwrap();
    } else {
        println!("No MIDI input ports available.");
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(2000));
            loop {
                let off = |note: Note| MidiEvent::NoteOff {
                    channel: 0,
                    note,
                    velocity: 0,
                };
                let on = |note: Note| MidiEvent::NoteOn {
                    channel: 0,
                    note,
                    velocity: 127,
                };
                for i in [0, 4, 7, 4] {
                    midi_tx.send(on(Note::middle_c().transpose(i))).ok();
                    std::thread::sleep(Duration::from_millis(50));
                    midi_tx.send(off(Note::middle_c().transpose(i))).ok();
                    std::thread::sleep(Duration::from_millis(450));
                }
            }
        });
    }

    // Create a collector
    let collector = Collector::new();

    // Create the input stream
    let host = cpal::default_host();
    let device = host.default_input_device().unwrap();
    let config = device.default_input_config().unwrap();
    let (audio_in, stream) = AudioInput::new(device, &config.into(), 2048, &collector.handle());
    stream.play().unwrap();

    // Create the output stream
    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();
    let config = device.default_output_config().unwrap();
    let sample_rate = config.sample_rate();
    let (audio_out, stream) = AudioOutput::new(device, &config.into(), 2048, &collector.handle());
    stream.play().unwrap();

    // Create the audio engine
    let mut engine = AudioEngine::new();
    engine.test(audio_in, audio_out);

    // Configure the audio engine
    engine.set_sample_rate(sample_rate.0);

    // Processing loop
    loop {
        let mut events = vec![];
        while let Ok(event) = midi_rx.try_recv() {
            events.push((0, event));
        }
        engine.process(256);
    }
}
