fn main() {
    println!("Building...");
    std::process::Command::new("cargo")
        .arg("build")
        .output()
        .unwrap();

    println!("Starting dev_setup.");
    let mut dev_setup = std::process::Command::new("cargo")
        .args(&["run", "--bin", "dev_setup"])
        .spawn()
        .unwrap();
    std::thread::sleep(std::time::Duration::from_secs(1));

    println!("Starting olivia backend.");
    let mut backend = std::process::Command::new("cargo")
        .env("RUST_LOG", "INFO")
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .args(&["run", "--bin", "olivia_backend"])
        .spawn()
        .unwrap();
    std::thread::sleep(std::time::Duration::from_secs(10));

    println!("Creating integration test JACK client.");
    let (client, _) = jack::Client::new(
        "olivia_integration_test",
        jack::ClientOptions::NO_START_SERVER,
    )
    .unwrap();
    for port in client.ports(None, None, jack::PortFlags::empty()).iter() {
        println!("Found port {}.", port);
    }

    println!("Midi can't be tested over CI so assuming that it works well.");
    let olivia_outputs = [
        client.port_by_name("olivia:output_l").unwrap(),
        client.port_by_name("olivia:output_r").unwrap(),
    ];
    let dev_playback = [
        client
            .port_by_name("olivia_dev_playback:playback_1")
            .unwrap(),
        client
            .port_by_name("olivia_dev_playback:playback_2")
            .unwrap(),
    ];
    for (i, o) in olivia_outputs.iter().zip(dev_playback.iter()) {
        println!(
            "Testing that {} is connected to {}.",
            i.name().unwrap(),
            o.name().unwrap()
        );
        let is_connected = i.is_connected_to(o.name().unwrap().as_str()).unwrap();
        assert!(is_connected);
    }

    println!("Tests completed OK!");
    dev_setup.kill().ok();
    backend.kill().ok();
}
