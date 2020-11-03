#[macro_use]
extern crate log;

mod adapter;

fn main() {
    env_logger::init();

    info!("Creating Olivia processor.");
    let processor = olivia_core::processor::Processor::new();

    info!("Running Olivia with JACK backend.");
    run_with_jack(processor);
}

fn run_with_jack(processor: olivia_core::processor::Processor) {
    adapter::jack::initialize_logging();
    let (client, status) =
        jack::Client::new("olivia", jack::ClientOptions::NO_START_SERVER).unwrap();
    info!(
        "Opened JACK client {} with status {:?}.",
        client.name(),
        status
    );

    info!("Creating JACK processor.");
    let jack_processor = adapter::jack::Processor::new(&client, processor).unwrap();

    info!("Starting JACK audio processing loop.");
    let _active_client = client.activate_async((), jack_processor).unwrap();
}
