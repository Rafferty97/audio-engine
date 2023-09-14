use crate::midi::MidiEvent;
use crate::note::Note;
use crate::processor::{
    AudioOutput, Autopan, Chord, Delay, Gain, Pipeline, Processor, ProcessorData, Saturator,
};
use crate::synth::{oscillators, Synth, SynthOpts, VoiceOpts};
use basedrop::Collector;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use midir::{Ignore, MidiInput};
use processor::AudioInput;
use std::io::{stdout, Write};
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
    let (midi_tx, midi_rx) = std::sync::mpsc::channel::<MidiEvent>();
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

    // Create the audio processors
    let synth = Synth::new(SynthOpts {
        num_voices: 16,
        voice_opts: VoiceOpts {
            wave: oscillators::saw,
            attack: 0.05,
            decay: 0.05,
            sustain: 0.8,
            release: 0.25,
        },
    });

    let mut chord = Chord::new();
    // chord.set_chord(0b10001001);
    chord.set_chord(0x10010001);

    let mut autopan = Autopan::new();
    autopan.set_frequency(2.0);
    autopan.set_amount(1.8);

    let mut gain = Gain::new();
    gain.set_gain(-9.0);

    let mut delay = Delay::new();
    delay.set_delay(0.125);
    delay.set_feedback(0.75);
    delay.set_ping_pong(true);

    let saturator = Saturator::new(|s| s.clamp(-1.0, 1.0));

    // Create the audio engine
    let mut engine = Pipeline::new([
        // Box::new(synth) as Box<dyn Processor + Send>,
        // Box::new(chord) as Box<dyn Processor + Send>,
        Box::new(audio_in) as Box<dyn Processor + Send>,
        Box::new(gain) as Box<dyn Processor + Send>,
        Box::new(delay) as Box<dyn Processor + Send>,
        // Box::new(autopan) as Box<dyn Processor + Send>,
        Box::new(saturator) as Box<dyn Processor + Send>,
        Box::new(audio_out) as Box<dyn Processor + Send>,
    ]);

    // Configure the audio engine
    engine.set_sample_rate(sample_rate.0);

    // Processing loop
    loop {
        let mut events = vec![];
        while let Ok(event) = midi_rx.try_recv() {
            events.push((0, event));
        }
        let mut midi_out = Vec::new();
        let mut left_out = [0.0; 256];
        let mut right_out = [0.0; 256];
        engine.process(ProcessorData {
            midi_in: &events,
            midi_out: &mut midi_out,
            samples: left_out.len(),
            audio_in: &[],
            audio_out: &mut [&mut left_out, &mut right_out],
        });
    }
}
