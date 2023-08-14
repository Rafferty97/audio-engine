use crate::convert::interleave_stereo;
use crate::midi::MidiEvent;
use crate::note::Note;
use crate::processor::{Autopan, Pipeline, Processor, ProcessorData, Saturator};
use crate::synth::{oscillators, Synth, SynthOpts, VoiceOpts};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::StreamConfig;
use midir::{Ignore, MidiInput};
use std::io::{stdin, stdout, Write};
use std::time::Duration;

mod constants;
mod convert;
mod midi;
mod note;
mod processor;
mod synth;

fn main() {
    // Get the default output device.
    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();
    let config = device.default_output_config().unwrap();

    // Set up the MIDI input interface.
    let mut midi_in = MidiInput::new("MIDI input").unwrap();
    midi_in.ignore(Ignore::ActiveSense);

    // MIDI channel
    let (midi_tx, midi_rx) = std::sync::mpsc::channel::<MidiEvent>();

    // List available input ports.
    let in_ports = midi_in.ports();
    let _connection;
    if !in_ports.is_empty() {
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
        let callback = move |_, message: &[u8], _: &mut ()| {
            let event = MidiEvent::from_raw(message);
            if event.is_invalid() {
                return;
            }
            midi_tx.send(event).ok();
        };

        // Connect to the selected MIDI input port.
        _connection = midi_in
            .connect(&input_port, "midi-read-connection", callback, ())
            .unwrap();
    } else {
        println!("No MIDI input ports available.");
        std::thread::spawn(move || loop {
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
            midi_tx.send(on(Note::middle_c())).ok();
            std::thread::sleep(Duration::from_millis(250));
            midi_tx.send(on(Note::middle_c().transpose(4))).ok();
            std::thread::sleep(Duration::from_millis(250));
            midi_tx.send(on(Note::middle_c().transpose(7))).ok();
            std::thread::sleep(Duration::from_millis(500));
            midi_tx.send(off(Note::middle_c())).ok();
            midi_tx.send(off(Note::middle_c().transpose(4))).ok();
            midi_tx.send(off(Note::middle_c().transpose(7))).ok();
            std::thread::sleep(Duration::from_millis(1000));
        });
    }

    // Create the audio engine
    let synth = Synth::new(SynthOpts {
        num_voices: 16,
        voice_opts: VoiceOpts {
            wave: oscillators::sine,
            attack: 0.05,
            decay: 0.05,
            sustain: 1.0,
            release: 0.05,
        },
    });
    let autopan = Autopan::new(1.0);
    let saturator = Saturator::new(|s| s.abs().powf(0.9) * s.signum());
    let engine = Pipeline::new([
        Box::new(synth) as Box<dyn Processor + Send>,
        Box::new(saturator) as Box<dyn Processor + Send>,
        Box::new(autopan) as Box<dyn Processor + Send>,
    ]);
    let mut engine: Box<dyn Processor + Send> = Box::new(engine);

    // Create the stream
    let config = StreamConfig {
        buffer_size: cpal::BufferSize::Fixed(256),
        channels: 2,
        sample_rate: config.sample_rate(),
    };

    // Configure the audio engine
    engine.set_sample_rate(config.sample_rate.0);

    let stream = device
        .build_output_stream(
            &config,
            move |buffer: &mut [f32], _| {
                let mut events = vec![];
                while let Ok(event) = midi_rx.try_recv() {
                    events.push((0, event));
                }
                let mut midi_out = Vec::new();
                let mut left = vec![0.0; buffer.len() / 2];
                let mut right = vec![0.0; buffer.len() / 2];
                engine.process(ProcessorData {
                    audio_in: &[],
                    audio_out: &mut [&mut left, &mut right],
                    midi_in: &events,
                    midi_out: &mut midi_out,
                });
                // Turn mono into stereo
                interleave_stereo(&left, &right, buffer);
            },
            |err| {
                eprintln!("{:?}", err);
            },
            None,
        )
        .unwrap();

    stream.play().unwrap();

    // Keep the program running.
    println!("Press Enter to exit...");
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
}
