#[macro_use]
extern crate log;

mod adapter;
mod controller;
mod plugin_registry;
mod plugin_factory;

fn main() {
    env_logger::init();

    info!("Loading audio plugins.");
    let plugin_factory = plugin_registry::new_plugin_factory();

    info!("Creating Olivia processor.");
    let (controller, processor) = controller::Controller::new(plugin_factory);

    info!("Running Olivia with JACK backend.");
    run_with_jack(controller, processor);
}

fn run_with_jack(mut controller: controller::Controller, processor: controller::Processor) {
    adapter::jack::initialize_logging();
    let (client, status) =
        jack::Client::new("olivia", jack::ClientOptions::NO_START_SERVER).unwrap();
    info!(
        "Opened JACK client {} with status {:?}.",
        client.name(),
        status
    );

    info!("Adding empty track \"{}\".", "Track 01");
    let buffer_size = client.buffer_size() as usize;
    controller.add_track("Track 01".to_string(), "builtin_silence", buffer_size);

    info!("Creating JACK processor.");
    let jack_processor = adapter::jack::Processor::new(&client, processor).unwrap();

    info!("Starting JACK audio processing loop.");
    let _active_client = client.activate_async((), jack_processor).unwrap();
}
