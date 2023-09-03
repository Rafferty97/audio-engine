use crate::audio::ring::RingBuffer;
use crate::convert::{interleave_stereo, uninterleave_stereo};
use crate::midi::MidiEvent;
use crate::note::Note;
use crate::processor::{
    Autopan, Chord, Delay, Gain, Pipeline, Processor, ProcessorData, Saturator,
};
use crate::synth::{oscillators, Synth, SynthOpts, VoiceOpts};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::StreamConfig;
use midir::{Ignore, MidiInput};
use std::io::{stdin, stdout, Write};
use std::sync::Mutex;
use std::time::Duration;

mod audio;
mod convert;
mod midi;
mod note;
mod processor;
mod synth;

fn main() {
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

    // Create the audio engine
    let synth = Synth::new(SynthOpts {
        num_voices: 16,
        voice_opts: VoiceOpts {
            wave: oscillators::tri,
            attack: 0.05,
            decay: 0.05,
            sustain: 0.8,
            release: 0.25,
        },
    });
    let mut chord = Chord::new();
    // chord.set_chord(0b10001001);
    chord.set_chord(0x10010001);
    let mut autopan = Autopan::new(1.0);
    autopan.set_amount(0.2);
    let mut gain = Gain::new();
    gain.set_gain(-6.0);
    let mut delay = Delay::new();
    delay.set_delay(0.15);
    delay.set_feedback(0.6);
    let saturator = Saturator::new(|s| s.clamp(-1.0, 1.0));
    let engine = Pipeline::new([
        // Box::new(chord) as Box<dyn Processor + Send>,
        // Box::new(synth) as Box<dyn Processor + Send>,
        Box::new(gain) as Box<dyn Processor + Send>,
        // Box::new(delay) as Box<dyn Processor + Send>,
        // Box::new(autopan) as Box<dyn Processor + Send>,
        Box::new(saturator) as Box<dyn Processor + Send>,
        Box::new(delay) as Box<dyn Processor + Send>,
    ]);
    let mut engine: Box<dyn Processor + Send> = Box::new(engine);

    // Ring buffer
    let left_buffer = Mutex::new(RingBuffer::new(4096));
    let left_buffer = &*Box::leak(Box::new(left_buffer));
    let right_buffer = Mutex::new(RingBuffer::new(4096));
    let right_buffer = &*Box::leak(Box::new(right_buffer));

    // Create the input stream
    let host = cpal::default_host();
    let device = host.default_input_device().unwrap();
    let config = device.default_input_config().unwrap();
    let stream = device
        .build_input_stream(
            &config.into(),
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                // Handle audio data here.
                let mut left = vec![0.0; data.len() / 2];
                let mut right = vec![0.0; data.len() / 2];
                uninterleave_stereo(data, &mut left, &mut right);
                left_buffer.lock().unwrap().write(&left);
                right_buffer.lock().unwrap().write(&right);
            },
            move |err| {
                eprintln!("an error occurred on stream: {}", err);
            },
            None,
        )
        .unwrap();
    stream.play().unwrap();

    // Create the output stream
    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();
    let config = device.default_output_config().unwrap();
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
            move |data: &mut [f32], _| {
                let mut events = vec![];
                while let Ok(event) = midi_rx.try_recv() {
                    events.push((0, event));
                }
                let mut midi_out = Vec::new();
                let mut left_in = vec![0.0; data.len() / 2];
                let mut right_in = vec![0.0; data.len() / 2];
                left_buffer.lock().unwrap().read(&mut left_in);
                right_buffer.lock().unwrap().read(&mut right_in);
                let mut left_out = vec![0.0; data.len() / 2];
                let mut right_out = vec![0.0; data.len() / 2];
                engine.process(ProcessorData {
                    midi_in: &events,
                    midi_out: &mut midi_out,
                    samples: left_in.len(),
                    audio_in: &[&left_in, &right_in],
                    audio_out: &mut [&mut left_out, &mut right_out],
                });
                // Turn mono into stereo
                interleave_stereo(&left_out, &right_out, data);
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
