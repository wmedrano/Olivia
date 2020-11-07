use crate::plugin::PluginInstance;
use crate::TimedMidi;

#[derive(Debug)]
pub struct Silence;

impl PluginInstance for Silence {
    fn process(&mut self, _: &[TimedMidi<'_>], out_left: &mut [f32], out_right: &mut [f32]) {
        zero_buffer(out_left);
        zero_buffer(out_right);
    }
}

fn zero_buffer(b: &mut [f32]) {
    for o in b.iter_mut() {
        *o = 0.0;
    }
}
