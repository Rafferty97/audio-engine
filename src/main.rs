#![allow(unused)]

use crate::midi::MidiEvent;
use crate::note::Note;
use crate::processor::AudioOutput;
use basedrop::Collector;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use engine::AudioEngine;
use processor::{AudioInput, MidiInput};
use std::time::Duration;

mod audio;
mod convert;
mod engine;
mod midi;
mod note;
mod processor;
mod synth;

fn main() {
    // Create a collector
    let collector = Collector::new();

    // Create the MIDI input processor
    let (midi_in, tx) = MidiInput::new();
    start_midi(tx);

    // Create the audio input processor
    let host = cpal::default_host();
    let device = host.default_input_device().unwrap();
    let config = device.default_input_config().unwrap();
    let (audio_in, stream) =
        AudioInput::from_cpal(device, &config.into(), 2048, &collector.handle());
    stream.play().unwrap();

    // Create the audio output processor
    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();
    let config = device.default_output_config().unwrap();
    let sample_rate = config.sample_rate();
    let (audio_out, stream) =
        AudioOutput::from_cpal(device, &config.into(), 2048, &collector.handle());
    stream.play().unwrap();

    // Create the audio engine
    let mut engine = AudioEngine::new();
    let delay = engine.test(audio_in, audio_out, midi_in);

    // Configure the audio engine
    engine.set_sample_rate(sample_rate.0);

    // Processing loop
    let mut i = 0;
    let mut d = 0.4;
    loop {
        engine.process(256);

        i += 256;
        if i > 4 * sample_rate.0 {
            d = 0.6 - d;
            engine.get_device_mut(delay).set_parameter(0, d);
            i = 0;
        }
    }
}

fn start_midi(tx: Box<dyn Fn(MidiEvent) + Send>) {
    let mut midi_in = midir::MidiInput::new("MIDI input").unwrap();
    midi_in.ignore(midir::Ignore::ActiveSense);
    let in_ports = midi_in.ports();

    let _connection;
    if !in_ports.is_empty() {
        // Create a callback to handle incoming MIDI messages
        let callback = move |_, message: &[u8], _: &mut ()| {
            let event = MidiEvent::from_raw(message);
            if event.is_invalid() {
                return;
            }
            tx(event);
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
                    tx(on(Note::middle_c().transpose(i)));
                    std::thread::sleep(Duration::from_millis(50));
                    tx(off(Note::middle_c().transpose(i)));
                    std::thread::sleep(Duration::from_millis(450));
                }
            }
        });
    }
}
