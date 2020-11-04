use crate::plugin;
use crate::TimedMidi;

#[derive(Debug)]
pub struct Processor {
    tracks: Vec<Track>,
    volume: f32,
}

impl Processor {
    pub fn new() -> Processor {
        Processor {
            tracks: Vec::with_capacity(1024),
            volume: 1.0,
        }
    }

    pub fn process(&mut self, midi: &[TimedMidi<'_>], out_left: &mut [f32], out_right: &mut [f32]) {
        zero_buffer(out_left);
        zero_buffer(out_right);

        let master_volume = self.volume;
        for track in self.tracks_mut() {
            track.process(midi);
            mix_into(out_left, &track.out_left, track.volume * master_volume);
            mix_into(out_right, &track.out_right, track.volume * master_volume);
        }
    }

    pub fn tracks_mut(&mut self) -> impl Iterator<Item = &'_ mut Track> {
        self.tracks.iter_mut()
    }

    pub fn add_track(&mut self, track: Track) {
        self.tracks.push(track);
    }

    pub fn remove_track(&mut self, index: usize) {
        self.tracks.remove(index);
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }
}

impl Default for Processor {
    fn default() -> Processor {
        Processor::new()
    }
}

#[derive(Debug)]
pub struct Track {
    plugins: Vec<Box<dyn plugin::PluginInstance>>,
    volume: f32,
    out_left: Vec<f32>,
    out_right: Vec<f32>,
}

impl Track {
    pub fn new(plugin: Box<dyn plugin::PluginInstance>, buffer_size: usize, volume: f32) -> Track {
        let mut plugins = Vec::with_capacity(128);
        plugins.push(plugin);
        Track {
            plugins,
            volume,
            out_left: vec![0.0; buffer_size],
            out_right: vec![0.0; buffer_size],
        }
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }

    fn process(&mut self, midi: &[TimedMidi<'_>]) {
        for plugin in self.plugins.iter_mut() {
            plugin.process(midi, &mut self.out_left, &mut self.out_right);
        }
    }
}

fn zero_buffer(b: &mut [f32]) {
    for o in b.iter_mut() {
        *o = 0.0;
    }
}

fn mix_into(dst: &mut [f32], src: &[f32], volume: f32) {
    for (o, i) in dst.iter_mut().zip(src.iter().cloned()) {
        *o += i * volume;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::PluginInstance;

    #[derive(Debug)]
    struct OnePluginInstance;
    impl PluginInstance for OnePluginInstance {
        fn process(&mut self, _: &[TimedMidi<'_>], out_left: &mut [f32], out_right: &mut [f32]) {
            fill_buffer(out_left, 1.0);
            fill_buffer(out_right, 1.0);
        }
    }

    fn fill_buffer(b: &mut [f32], scalar: f32) {
        for o in b.iter_mut() {
            *o = scalar;
        }
    }

    #[test]
    fn outputs_are_cleared() {
        let mut left = [1.0, 2.0];
        let mut right = [3.0, 4.0];

        let mut p = Processor::new();
        assert_ne!([left, right], [[0.0, 0.0], [0.0, 0.0]]);
        p.process(&[], &mut left, &mut right);
        assert_eq!([left, right], [[0.0, 0.0], [0.0, 0.0]]);
    }

    #[test]
    fn tracks_are_played() {
        let mut p = Processor::new();
        p.add_track(Track::new(Box::new(OnePluginInstance), 1024, 0.5));
        p.add_track(Track::new(Box::new(OnePluginInstance), 1024, 0.25));

        let mut left = [0.0; 2];
        let mut right = [0.0; 2];
        p.process(&[], &mut left, &mut right);

        assert_eq!([left, right], [[0.75, 0.75], [0.75, 0.75]])
    }

    #[test]
    fn tracks_are_removed() {
        let mut p = Processor::new();
        p.add_track(Track::new(Box::new(OnePluginInstance), 1024, 0.5));
        p.add_track(Track::new(Box::new(OnePluginInstance), 1024, 0.25));
        p.remove_track(0);

        let mut left = [0.0; 2];
        let mut right = [0.0; 2];
        p.process(&[], &mut left, &mut right);

        assert_eq!([left, right], [[0.25, 0.25], [0.25, 0.25]])
    }

    #[test]
    fn tracks_can_set_volume() {
        let mut p = Processor::new();
        let mut t = Track::new(Box::new(OnePluginInstance), 1024, 1.0);
        t.set_volume(0.5);
        p.add_track(t);

        let mut left = [0.0; 2];
        let mut right = [0.0; 2];
        p.process(&[], &mut left, &mut right);

        assert_eq!([left, right], [[0.5, 0.5], [0.5, 0.5]])
    }

    #[test]
    fn processor_can_set_volume() {
        let mut p = Processor::new();
        p.add_track(Track::new(Box::new(OnePluginInstance), 1024, 1.0));
        p.set_volume(2.0);

        let mut left = [0.0; 2];
        let mut right = [0.0; 2];
        p.process(&[], &mut left, &mut right);

        assert_eq!([left, right], [[2.0, 2.0], [2.0, 2.0]])
    }
}
