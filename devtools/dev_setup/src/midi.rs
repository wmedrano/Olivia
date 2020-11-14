use lazy_static::lazy_static;

lazy_static! {
    static ref PORT_MIDI: portmidi::PortMidi = portmidi::PortMidi::new().unwrap();
}

// run starts up a JACK client that passes through information from JACK
// devices.
pub fn run() {
    let (client, status) = jack::Client::new(
        "olivia_dev_midi_input",
        jack::ClientOptions::NO_START_SERVER,
    )
    .unwrap();
    println!(
        "Started olivia_dev client {} with status {:?}.",
        client.name(),
        status
    );

    const MIDI_BUFFER_SIZE: usize = 1024;
    let devices = PORT_MIDI.devices().unwrap();
    let mut inputs: Vec<_> = devices
        .iter()
        .filter(|d| d.is_input())
        .map(|d| PORT_MIDI.input_port(d.clone(), MIDI_BUFFER_SIZE).unwrap())
        .collect();

    let mut outputs: Vec<_> = inputs
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let name = format!("midi_{}", i + 1);
            println!("Using MIDI device {} as {}.", p.device().name(), name);
            name
        })
        .map(|n| client.register_port(&n, jack::MidiOut::default()).unwrap())
        .collect();

    let process = jack::ClosureProcessHandler::new(
        move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            let inputs = inputs.iter_mut();
            let outputs = outputs.iter_mut().map(|p| p.writer(ps));
            for (input_port, mut output_port) in inputs.zip(outputs) {
                loop {
                    match input_port.read() {
                        Ok(Some(m)) => {
                            let bytes = [
                                m.message.status,
                                m.message.data1,
                                m.message.data2,
                                m.message.data3,
                            ];
                            let raw_midi = jack::RawMidi {
                                time: 0,
                                bytes: &bytes,
                            };
                            output_port.write(&raw_midi).unwrap();
                        }
                        _ => break,
                    }
                }
                let raw_midi = jack::RawMidi {
                    time: 0,
                    bytes: &[],
                };
                output_port.write(&raw_midi).unwrap();
            }
            jack::Control::Continue
        },
    );

    let active_client = client.activate_async((), process).unwrap();
    std::thread::park();
    active_client.deactivate().unwrap();
}
