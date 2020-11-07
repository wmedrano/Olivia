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

    // Uncomment the backend you want to use.
    let backend = adapter::jack::JackBackend::new(processor).unwrap();
    // let backend = adapter::dummy_io::DummyBackend(processor);
    info!("Running Olivia with {} backend.", backend.name());
    controller.set_buffer_size(backend.buffer_size());
    let _process_thread = std::thread::spawn(move || {
        let backend_name = backend.name();
        backend.run_process_loop();
        panic!("IO backend {} terminated unexpectedly.", backend_name);
    });

    info!("Creating initial track.");
    let initial_track = controller::Track{
        name: "Track 01".to_string(),
        volume: 1.0,
        plugin_instances: Vec::new(),
    };
    controller.add_track(initial_track);

    info!("Starting actix webserver.");
    let controller = std::sync::Arc::new(std::sync::Mutex::new(Some(controller)));
    actix_web::HttpServer::new(move || {
        let controller_arc = controller.clone();
        let mut some_controller = controller_arc.lock().unwrap();
        // Since we only have a single worker thread, we should only ever take the value once ensuring that some
        // controller is indeed Some(controller) rather than None by this point.
        let handler = adapter::actix_server::Handler::new(some_controller.take().unwrap());
        actix_web::App::new()
            .data(std::sync::Mutex::new(handler))
            .route(
                "/plugins",
                actix_web::web::get().to(adapter::actix_server::get_plugins),
            )
            .route(
                "/tracks",
                actix_web::web::get().to(adapter::actix_server::get_tracks),
            )
    })
    .workers(1)
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
