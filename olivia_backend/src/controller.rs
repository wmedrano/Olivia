use crate::plugin_factory;
use olivia_core::TimedMidi;

enum Command {
    AddTrack(olivia_core::processor::Track),
}

pub struct Track {
    pub name: String,
    pub volume: f32,
}

pub struct Controller {
    tracks: Vec<Track>,
    plugin_factory: plugin_factory::PluginFactory,
    commands: crossbeam::channel::Sender<Command>,
}

impl Controller {
    pub fn new(plugin_factory: plugin_factory::PluginFactory) -> (Controller, Processor) {
        let command_queue_size = 1_000_000;
        let (tx, rx) = crossbeam::channel::bounded(command_queue_size);
        let controller = Controller {
            tracks: Vec::new(),
            plugin_factory,
            commands: tx,
        };
        let processor = Processor {
            inner: olivia_core::processor::Processor::new(),
            commands: rx,
        };
        (controller, processor)
    }

    pub fn add_track(&mut self, track_name: String, plugin_id: &str, buffer_size: usize) {
        let track = Track {
            name: track_name,
            volume: 1.0,
        };
        let core_track = self
            .plugin_factory
            .build_track(plugin_id, buffer_size)
            .unwrap();
        self.commands.send(Command::AddTrack(core_track)).unwrap();
        self.tracks.push(track);
    }
}

pub struct Processor {
    inner: olivia_core::processor::Processor,
    commands: crossbeam::channel::Receiver<Command>,
}

impl Processor {
    pub fn process(&mut self, midi: &[TimedMidi], out_left: &mut [f32], out_right: &mut [f32]) {
        for command in self.commands.try_iter() {
            match command {
                Command::AddTrack(t) => self.inner.add_track(t),
            }
        }
        self.inner.process(midi, out_left, out_right);
    }
}
