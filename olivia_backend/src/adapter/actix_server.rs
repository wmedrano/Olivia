use crate::controller::Controller;
use std::sync::Mutex;

pub struct Handler {
    controller: Controller,
}

impl Handler {
    pub fn new(controller: Controller) -> Handler {
        Handler { controller }
    }

    pub fn plugins(&self) -> Vec<crate::plugin_factory::PluginMetadata> {
        self.controller
            .plugin_factory()
            .metadata()
            .cloned()
            .collect()
    }
}

pub async fn get_plugins(data: actix_web::web::Data<Mutex<Handler>>) -> impl actix_web::Responder {
    let handler = data.lock().unwrap();
    actix_web::web::Json(handler.plugins())
}
