use crate::plugin_factory;
use olivia_core::TimedMidi;
use plugin_factory::PluginFactory;
use std::collections::HashMap;

#[derive(
    Copy, Clone, Debug, Eq, Hash, PartialEq, Ord, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub struct IntId(pub usize);

enum Command {
    AddTrack(olivia_core::processor::Track),
    DeleteTrack(usize),
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct PluginInstance {
    pub id: IntId,
    pub plugin_id: String,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Track {
    pub id: IntId,
    pub name: String,
    pub volume: f32,
    pub plugin_instances: Vec<IntId>,
}

pub struct Controller {
    // Metadata for all tracks.
    tracks: Vec<Track>,
    // Metadata for all plugin instances.
    plugin_instances: Vec<PluginInstance>,
    // Plugin instances that don't belong to any tracks.
    unowned_plugin_instances: HashMap<IntId, Box<dyn olivia_core::plugin::PluginInstance>>,
    // Factory containing plugin metadata as well as methods for building plugin
    // instances.
    plugin_factory: PluginFactory,
    // Buffer size.
    buffer_size: usize,
    // Channel to send commands to audio processor.
    commands: crossbeam::channel::Sender<Command>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ControllerError {
    BufferSizeHasNotBeenSet,
    PluginInstanceAlreadyExists(IntId, PluginInstance),
    FailedToBuildPlugin(plugin_factory::PluginBuilderError),
    TrackAlreadyExists(IntId, Track),
    TrackDoesNotExist(IntId),
    TrackReferencesNonExistantPluginInstance {
        track_id: IntId,
        plugin_instance_id: IntId,
    },
    MovingPluginInstancesBetweenTracksNotImplemented {
        new_track_id: IntId,
        current_track_id: IntId,
        plugin_instance_id: IntId,
    },
}

impl std::error::Error for ControllerError {}

impl std::fmt::Display for ControllerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Controller {
    pub fn new(plugin_factory: PluginFactory) -> (Controller, Processor) {
        let command_queue_size = 1_000_000;
        let (tx, rx) = crossbeam::channel::bounded(command_queue_size);
        let controller = Controller {
            tracks: Vec::new(),
            plugin_instances: Vec::new(),
            unowned_plugin_instances: HashMap::new(),
            plugin_factory,
            buffer_size: 0,
            commands: tx,
        };
        let processor = Processor {
            inner: olivia_core::processor::Processor::new(),
            commands: rx,
        };
        (controller, processor)
    }

    pub fn set_buffer_size(&mut self, buffer_size: usize) {
        self.buffer_size = buffer_size;
    }

    pub fn tracks(&self) -> impl Iterator<Item = &'_ Track> {
        self.tracks.iter()
    }

    pub fn track_by_id(&self, id: IntId) -> Option<&Track> {
        self.tracks().find(|t| t.id == id)
    }

    pub fn add_track(&mut self, track: Track) -> Result<(), ControllerError> {
        if self.buffer_size == 0 {
            error!(
                "Attempted to create track without setting buffer size. Track: {:?}.",
                track
            );
            return Err(ControllerError::BufferSizeHasNotBeenSet);
        }
        if let Some(t) = self.track_by_id(track.id) {
            return Err(ControllerError::TrackAlreadyExists(t.id, t.clone()));
        }
        for plugin_instance in track.plugin_instances.iter() {
            if self.plugin_instance_by_id(*plugin_instance).is_none() {
                return Err(ControllerError::TrackReferencesNonExistantPluginInstance {
                    track_id: track.id,
                    plugin_instance_id: plugin_instance.clone(),
                });
            }
            for t in self.tracks() {
                if t.plugin_instances.contains(plugin_instance) {
                    return Err(
                        ControllerError::MovingPluginInstancesBetweenTracksNotImplemented {
                            new_track_id: track.id,
                            current_track_id: t.id,
                            plugin_instance_id: *plugin_instance,
                        },
                    );
                }
            }
        }

        info!("Creating track \"{}\".", track.name);
        let mut core_track = olivia_core::processor::Track::new(self.buffer_size, 1.0);
        for plugin_instance in track.plugin_instances.iter() {
            core_track.add_plugin(
                self.unowned_plugin_instances
                    .remove(plugin_instance)
                    .unwrap(),
            );
        }
        self.commands.send(Command::AddTrack(core_track)).unwrap();
        self.tracks.push(track);
        Ok(())
    }

    pub fn delete_track(&mut self, id: IntId) -> Result<(), ControllerError> {
        let track_index = match self.tracks.iter().enumerate().find(|(_, t)| t.id == id) {
            Some((idx, _)) => idx,
            None => return Err(ControllerError::TrackDoesNotExist(id)),
        };
        for pid in self.tracks[track_index].plugin_instances.iter() {
            self.plugin_instances.retain(|p| p.id != *pid);
        }
        self.tracks.retain(|t| t.id != id);
        self.commands
            .send(Command::DeleteTrack(track_index))
            .unwrap();
        Ok(())
    }

    pub fn plugin_factory(&self) -> &PluginFactory {
        &self.plugin_factory
    }

    pub fn plugin_instances(&self) -> impl Iterator<Item = &'_ PluginInstance> {
        self.plugin_instances.iter()
    }

    pub fn plugin_instance_by_id(&self, id: IntId) -> Option<&PluginInstance> {
        self.plugin_instances().find(|p| p.id == id)
    }

    pub fn create_plugin_instance(
        &mut self,
        metadata: PluginInstance,
    ) -> Result<(), ControllerError> {
        if let Some(p) = self.plugin_instance_by_id(metadata.id) {
            return Err(ControllerError::PluginInstanceAlreadyExists(
                p.id,
                p.clone(),
            ));
        }
        let plugin_instance = self
            .plugin_factory
            .build(&metadata.plugin_id)
            .map_err(|e| ControllerError::FailedToBuildPlugin(e))?;
        self.unowned_plugin_instances
            .insert(metadata.id.clone(), plugin_instance);
        self.plugin_instances.push(metadata);
        Ok(())
    }
}

pub struct Processor {
    inner: olivia_core::processor::Processor,
    commands: crossbeam::channel::Receiver<Command>,
}

impl Processor {
    pub fn process(&mut self, midi: &[TimedMidi], out_left: &mut [f32], out_right: &mut [f32]) {
        self.handle_commands();
        self.inner.process(midi, out_left, out_right);
    }

    fn handle_commands(&mut self) {
        for command in self.commands.try_iter() {
            match command {
                Command::AddTrack(t) => self.inner.add_track(t),
                Command::DeleteTrack(track_index) => self.inner.delete_track(track_index),
            }
        }
    }
}
