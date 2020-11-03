#[macro_use]
extern crate log;

mod adapter;

fn main() {
    env_logger::init();
    
    let (client, status) = jack::Client::new("olivia", jack::ClientOptions::NO_START_SERVER).unwrap();
    info!(
        "Opened JACK client {} with status {:?}.",
        client.name(),
        status
    );

    info!("Creating JACK processor.");
    let processor = adapter::jack::Processor::new(&client).unwrap();

    info!("Starting JACK audio processing loop.");
    let _active_client = client.activate_async((), processor).unwrap();
}
