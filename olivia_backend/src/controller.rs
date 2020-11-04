use crate::plugin_factory;
use olivia_core::TimedMidi;

enum Command {
    AddTrack(olivia_core::processor::Track),
    RemoveTrack(usize),
    SetTrackVolume(usize, f32),
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

    pub fn remove_track(&mut self, index: usize) {
        if index < self.tracks.len() {
            self.tracks.remove(index);
            self.commands.send(Command::RemoveTrack(index)).unwrap();
        }
    }

    pub fn set_track_volume(&mut self, index: usize, volume: f32) {
        if index < self.tracks.len() {
            self.tracks[index].volume = volume;
            self.commands
                .send(Command::SetTrackVolume(index, volume))
                .unwrap();
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
                Command::SetTrackVolume(index, volume) => {
                    if let Some(t) = self.inner.tracks_mut().skip(index).next() {
                        t.set_volume(volume);
                    }
                }
            }
        }
        self.inner.process(midi, out_left, out_right);
    }
}
