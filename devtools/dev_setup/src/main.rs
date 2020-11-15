mod midi;
mod playback;

use jack::PortSpec;

fn main() {
    let mut server = run_server();
    ctrlc::set_handler(move || {
        let script_terminated_by_ctrl_c_code = 130;
        println!("Kill server.");
        server.kill().unwrap();
        std::process::exit(script_terminated_by_ctrl_c_code);
    })
    .unwrap();
    std::thread::sleep(std::time::Duration::from_millis(500));

    std::thread::spawn(|| playback::run());
    std::thread::sleep(std::time::Duration::from_millis(200));

    std::thread::spawn(|| midi::run());
    std::thread::sleep(std::time::Duration::from_millis(200));

    let port_connector = jack::Client::new(
        "olivia_dev_auto_connect",
        jack::ClientOptions::NO_START_SERVER,
    )
    .unwrap()
    .0;

    let backend_outputs = ["olivia:output_l", "olivia:output_r"];
    let midi_outputs_regexp = format!("{}:.*", midi::CLIENT_NAME);
    let playback_outputs: Vec<_> = playback::PLAYBACK_PORTS
        .iter()
        .map(|p| format!("{}:{}", playback::CLIENT_NAME, p))
        .collect();
    let do_connect = || {
        for (a, b) in backend_outputs.iter().zip(playback_outputs.iter()) {
            if let Err(e) = port_connector.connect_ports_by_name(a, b) {
                format!("Error connecting olivia backend to dev playback: {:?}", e);
            };
        }
        let midi_output_ports = port_connector.ports(
            Some(&midi_outputs_regexp),
            Some(jack::MidiOut::default().jack_port_type()),
            jack::PortFlags::empty(),
        );
        for p in midi_output_ports.iter() {
            if let Err(e) = port_connector.connect_ports_by_name(p, "olivia:midi_input") {
                format!("Error connecting dev midi to olivia backend: {:?}", e);
            };
        }
    };
    loop {
        let olivia_backend_is_running = port_connector.port_by_name("olivia:midi_input").is_some();
        if olivia_backend_is_running {
            do_connect();
        }
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}

fn run_server() -> std::process::Child {
    std::thread::sleep(std::time::Duration::from_millis(200));
    println!("Starting dummy server.");
    std::process::Command::new("jackd")
        .args(&["-ddummy", "-r44100", "-p2048"])
        .spawn()
        .unwrap()
}
