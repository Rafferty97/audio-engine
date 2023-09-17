fn start_midi(tx: Box<dyn Fn(MidiEvent) + Send>) {
    let mut midi_in = midir::MidiInput::new("MIDI input").unwrap();
    midi_in.ignore(midir::Ignore::ActiveSense);
    let in_ports = midi_in.ports();

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
        let c = midi_in
            .connect(&in_ports[0], "midi-read-connection", callback, ())
            .unwrap();
        Box::leak(Box::new(c));
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
