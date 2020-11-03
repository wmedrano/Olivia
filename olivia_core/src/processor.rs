use crate::plugin;
use crate::plugin::PluginInstance;
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

    pub fn tracks_mut(&mut self) -> impl Iterator<Item=&'_ mut Track> {
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

impl Processor {
    pub fn process(&mut self, midi: &[TimedMidi<'_>], out_left: &mut [f32], out_right: &mut [f32]) {
        let mut s = plugin::Silence;
        s.process(&[], out_left, out_right);

        for track in self.tracks_mut() {
            track.process(midi);
            for (dst, src) in out_left.iter_mut().zip(track.out_left.iter().cloned()) {
                *dst += src * track.volume;
            }
            for (dst, src) in out_right.iter_mut().zip(track.out_right.iter().cloned()) {
                *dst += src * track.volume;
            }
        }
        scale_buffer(out_left, self.volume);
        scale_buffer(out_right, self.volume);
    }
}

#[derive(Debug)]
pub struct Track {
    plugin: Box<dyn plugin::PluginInstance>,
    volume: f32,
    out_left: Vec<f32>,
    out_right: Vec<f32>,
}

impl Track {
    pub fn new(plugin: Box<dyn plugin::PluginInstance>, buffer_size: usize, volume: f32) -> Track {
        Track {
            plugin,
            volume,
            out_left: vec![0.0; buffer_size],
            out_right: vec![0.0; buffer_size],
        }
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }

    fn process(&mut self, midi: &[TimedMidi<'_>]) {
        self.plugin
            .process(midi, &mut self.out_left, &mut self.out_right);
    }
}

fn scale_buffer(b: &mut [f32], scalar: f32) {
    for o in b.iter_mut() {
        *o *= scalar;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
