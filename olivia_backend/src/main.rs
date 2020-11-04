#[macro_use]
extern crate log;

mod adapter;
mod controller;
mod io_backend;
mod plugin_factory;
mod plugin_registry;

use io_backend::IoBackend;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    info!("Loading audio plugins.");
    let plugin_factory = plugin_registry::new_plugin_factory();

    info!("Creating Olivia processor.");
    let (mut controller, processor) = controller::Controller::new(plugin_factory);

    info!("Running Olivia with JACK backend.");
    let backend = adapter::jack::JackBackend::new(processor).unwrap();
    let buffer_size = backend.buffer_size();
    let _process_thread = std::thread::spawn(move || {
        let backend_name = backend.name();
        backend.run_process_loop();
        panic!("IO backend {} terminated unexpectedly.", backend_name);
    });

    info!("Creating initial track.");
    controller.add_track("Track 01".to_string(), "builtin_silence", buffer_size);

    info!("Starting actix webserver.");
    actix_web::HttpServer::new(move || actix_web::App::new())
        .workers(1)
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
