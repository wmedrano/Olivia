use crate::plugin_factory;
use olivia_core::TimedMidi;

#[derive(
    Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub struct IntId(pub usize);

enum Command {
    AddTrack(olivia_core::processor::Track),
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct PluginInstance {
    id: usize,
    plugin_id: String,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct Track {
    pub id: IntId,
    pub name: String,
    pub volume: f32,
    pub plugin_instances: Vec<PluginInstance>,
}

pub struct Controller {
    // Metadata for all tracks.
    tracks: Vec<Track>,
    // Factory containing plugin metadata as well as methods for building plugin
    // instances.
    plugin_factory: plugin_factory::PluginFactory,
    // Buffer size.
    buffer_size: usize,
    // Channel to send commands to audio processor.
    commands: crossbeam::channel::Sender<Command>,
}

#[derive(Clone, Debug)]
pub enum ControllerError {
    PluginInstancesNotYetSupported(Track),
    TrackAlreadyExists(IntId, Track),
    BufferSizeHasNotBeenSet,
}

impl std::error::Error for ControllerError {}

impl std::fmt::Display for ControllerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Controller {
    pub fn new(plugin_factory: plugin_factory::PluginFactory) -> (Controller, Processor) {
        let command_queue_size = 1_000_000;
        let (tx, rx) = crossbeam::channel::bounded(command_queue_size);
        let controller = Controller {
            tracks: Vec::new(),
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

    pub fn add_track(&mut self, track: Track) -> Result<(), ControllerError> {
        if self.buffer_size == 0 {
            error!(
                "Attempted to create track without setting buffer size. Track: {:?}.",
                track
            );
            return Err(ControllerError::BufferSizeHasNotBeenSet);
        }
        if !track.plugin_instances.is_empty() {
            error!("Failed to create track {} because plugin instances are not yet supported. Track: {:?}.", track.name, track);
            return Err(ControllerError::PluginInstancesNotYetSupported(track));
        }
        if let Some(t) = self.track_by_id(track.id) {
            return Err(ControllerError::TrackAlreadyExists(t.id, t.clone()));
        }
        info!("Creating track \"{}\".", track.name);
        let core_track = olivia_core::processor::Track::new(self.buffer_size, 1.0);
        self.commands.send(Command::AddTrack(core_track)).unwrap();
        self.tracks.push(track);
        Ok(())
    }

    pub fn plugin_factory(&self) -> &plugin_factory::PluginFactory {
        &self.plugin_factory
    }

    pub fn tracks(&self) -> impl Iterator<Item = &'_ Track> {
        self.tracks.iter()
    }

    pub fn track_by_id(&self, id: IntId) -> Option<&Track> {
        self.tracks().find(|t| t.id == id)
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
            }
        }
    }
}
