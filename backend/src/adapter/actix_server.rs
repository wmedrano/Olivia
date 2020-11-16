use crate::controller::{Controller, IntId};
use std::convert::From;
use std::sync::Mutex;

#[derive(Clone, Debug, PartialEq)]
enum Error {
    GenericController(crate::controller::ControllerError),
    TrackNotFound(IntId),
    PluginInstanceNotFound(IntId),
    PluginInstanceUpdateNotImplemented,
    UpdatingTrackNotImplemented,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<crate::controller::ControllerError> for Error {
    fn from(e: crate::controller::ControllerError) -> Error {
        Error::GenericController(e)
    }
}

impl actix_web::error::ResponseError for Error {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Error::GenericController(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            Error::TrackNotFound(_) => actix_web::http::StatusCode::NOT_FOUND,
            Error::PluginInstanceNotFound(_) => actix_web::http::StatusCode::NOT_FOUND,
            Error::PluginInstanceUpdateNotImplemented => {
                actix_web::http::StatusCode::NOT_IMPLEMENTED
            }
            Error::UpdatingTrackNotImplemented => actix_web::http::StatusCode::NOT_IMPLEMENTED,
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

    fn controller(&self) -> &Controller {
        &self.controller
    }

    fn controller_mut(&mut self) -> &mut Controller {
        &mut self.controller
    }
}

pub async fn get_plugins(data: actix_web::web::Data<Mutex<Handler>>) -> impl actix_web::Responder {
    let handler = data.lock().unwrap();
    let plugins: Vec<_> = handler
        .controller()
        .plugin_factory()
        .metadata()
        .cloned()
        .collect();
    actix_web::web::Json(plugins)
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

pub async fn put_track(
    track_id: actix_web::web::Path<IntId>,
    mut track: actix_web::web::Json<crate::controller::Track>,
    data: actix_web::web::Data<Mutex<Handler>>,
) -> impl actix_web::Responder {
    let mut handler = data.lock().unwrap();
    if handler.controller().track_by_id(track_id.0).is_some() {
        return Err(Error::UpdatingTrackNotImplemented);
    }
    handler.controller_mut().add_track(track.0.clone())?;
    track.0.id = track_id.0;
    Ok(actix_web::web::Json(track.0))
}

pub async fn delete_track(
    track_id: actix_web::web::Path<IntId>,
    data: actix_web::web::Data<Mutex<Handler>>,
) -> impl actix_web::Responder {
    let mut handler = data.lock().unwrap();
    if handler.controller().track_by_id(track_id.0).is_none() {
        return Err(Error::TrackNotFound(track_id.0));
    }
    handler.controller_mut().delete_track(track_id.0)?;
    Ok(actix_web::web::Json(""))
}

pub async fn get_plugin_instances(
    data: actix_web::web::Data<Mutex<Handler>>,
) -> impl actix_web::Responder {
    let handler = data.lock().unwrap();
    let plugin_instances: Vec<_> = handler.controller().plugin_instances().cloned().collect();
    actix_web::web::Json(plugin_instances)
}

pub async fn get_plugin_instance(
    plugin_instance_id: actix_web::web::Path<IntId>,
    data: actix_web::web::Data<Mutex<Handler>>,
) -> impl actix_web::Responder {
    let handler = data.lock().unwrap();
    match handler
        .controller()
        .plugin_instance_by_id(plugin_instance_id.0)
    {
        Some(p) => Ok(actix_web::web::Json(p.clone())),
        None => Err(Error::PluginInstanceNotFound(plugin_instance_id.0)),
    }
}

pub async fn put_plugin_instance(
    plugin_instance_id: actix_web::web::Path<IntId>,
    mut plugin_instance: actix_web::web::Json<crate::controller::PluginInstance>,
    data: actix_web::web::Data<Mutex<Handler>>,
) -> impl actix_web::Responder {
    let mut handler = data.lock().unwrap();
    plugin_instance.0.id = plugin_instance_id.0;
    if handler
        .controller
        .plugin_instance_by_id(plugin_instance_id.0)
        .is_some()
    {
        return Err(Error::PluginInstanceUpdateNotImplemented);
    }
    match handler
        .controller_mut()
        .create_plugin_instance(plugin_instance.0.clone())
    {
        Ok(()) => Ok(actix_web::web::Json(plugin_instance.0)),
        Err(e) => Err(Error::GenericController(e)),
    }
}
