use crate::controller::{Controller, IntId};
use std::sync::Mutex;

#[derive(Clone, Debug, Eq, PartialEq)]
enum Error {
    TrackNotFound(IntId),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl actix_web::error::ResponseError for Error {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Error::TrackNotFound(_) => actix_web::http::StatusCode::NOT_FOUND,
        }
    }
}

pub struct Handler {
    controller: Controller,
}

impl Handler {
    pub fn new(controller: Controller) -> Handler {
        Handler { controller }
    }

    fn plugins(&self) -> Vec<crate::plugin_factory::PluginMetadata> {
        self.controller
            .plugin_factory()
            .metadata()
            .cloned()
            .collect()
    }

    fn controller(&self) -> &Controller {
        &self.controller
    }
}

pub async fn get_plugins(data: actix_web::web::Data<Mutex<Handler>>) -> impl actix_web::Responder {
    let handler = data.lock().unwrap();
    actix_web::web::Json(handler.plugins())
}

pub async fn get_tracks(data: actix_web::web::Data<Mutex<Handler>>) -> impl actix_web::Responder {
    let handler = data.lock().unwrap();
    let tracks: Vec<_> = handler.controller().tracks().cloned().collect();
    actix_web::web::Json(tracks)
}

pub async fn get_track(
    track_id: actix_web::web::Path<IntId>,
    data: actix_web::web::Data<Mutex<Handler>>,
) -> impl actix_web::Responder {
    let handler = data.lock().unwrap();
    match handler.controller().track_by_id(track_id.0) {
        Some(t) => Ok(actix_web::web::Json(t.clone())),
        None => Err(Error::TrackNotFound(track_id.0)),
    }
}
