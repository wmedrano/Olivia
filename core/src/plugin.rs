use crate::TimedMidi;
pub trait PluginInstance: Send + std::fmt::Debug {
    fn process(&mut self, midi: &[TimedMidi<'_>], out_left: &mut [f32], out_right: &mut [f32]);
}
