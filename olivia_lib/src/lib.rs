#[macro_use]
extern crate log;

use olivia_core::TimedMidi;

pub mod plugin_factory;

enum Command {
    AddTrack(olivia_core::processor::Track),
    RemoveTrack(usize),
}

pub struct Track {
    pub name: String,
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
        let track = Track { name: track_name };
        let core_track = self
            .plugin_factory
            .build_track(plugin_id, buffer_size)
            .unwrap();
        self.commands.send(Command::AddTrack(core_track)).unwrap();
        self.tracks.push(track);
    }

    pub fn remove_track(&mut self, index: usize) {
        if index < self.tracks.len() {
            self.tracks.remove(index);
            self.commands.send(Command::RemoveTrack(index)).unwrap();
        }
    }

    pub fn tracks(&self) -> impl Iterator<Item = &'_ Track> {
        self.tracks.iter()
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
                Command::RemoveTrack(index) => self.inner.remove_track(index),
            }
        }
        self.inner.process(midi, out_left, out_right);
    }
}
