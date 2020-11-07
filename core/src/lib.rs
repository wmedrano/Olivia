pub mod example_plugin;
pub mod plugin;
pub mod processor;

pub struct TimedMidi<'a> {
    pub frame: usize,
    pub message: wmidi::MidiMessage<'a>,
}
