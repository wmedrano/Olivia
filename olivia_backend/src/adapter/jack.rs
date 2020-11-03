use std::convert::TryFrom;

pub struct Processor {
    midi_input: jack::Port<jack::MidiIn>,
    temp_midi_buffer: Vec<olivia_core::TimedMidi<'static>>,
    outputs: [jack::Port<jack::AudioOut>; 2],
    processor: olivia_lib::Processor,
}

impl Processor {
    pub fn new(
        client: &jack::Client,
        processor: olivia_lib::Processor,
    ) -> Result<Processor, jack::Error> {
        let midi_input = client.register_port("midi_input", jack::MidiIn::default())?;
        let outputs = [
            client.register_port("output_l", jack::AudioOut::default())?,
            client.register_port("output_r", jack::AudioOut::default())?,
        ];
        // This is a somewhat large but arbitrary number.
        let temp_midi_buffer_size = 1_000_000;
        info!(
            "Initializing midi with buffer size {}.",
            temp_midi_buffer_size
        );
        let temp_midi_buffer = Vec::with_capacity(temp_midi_buffer_size);
        Ok(Processor {
            midi_input,
            temp_midi_buffer,
            outputs,
            processor,
        })
    }
}

impl jack::ProcessHandler for Processor {
    fn process(&mut self, _: &jack::Client, ps: &jack::ProcessScope) -> jack::Control {
        self.temp_midi_buffer.clear();
        for raw_midi in self.midi_input.iter(ps) {
            if let Ok(Some(message)) = wmidi::MidiMessage::try_from(raw_midi.bytes)
                .map(wmidi::MidiMessage::drop_unowned_sysex)
            {
                self.temp_midi_buffer.push(olivia_core::TimedMidi {
                    frame: raw_midi.time as usize,
                    message,
                });
            }
        }
        let (out_left, out_right) = match &mut self.outputs {
            [left, right] => (left.as_mut_slice(ps), right.as_mut_slice(ps)),
        };
        self.processor
            .process(&self.temp_midi_buffer, out_left, out_right);
        jack::Control::Continue
    }
}

pub fn initialize_logging() {
    jack::set_error_callback(error_callback);
    jack::set_info_callback(info_callback);
}

fn error_callback(msg: &str) {
    error!("{}", msg);
}

fn info_callback(msg: &str) {
    info!("{}", msg);
}
