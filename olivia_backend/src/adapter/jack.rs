pub struct Processor {
    midi_input: jack::Port<jack::MidiIn>,
    outputs: [jack::Port<jack::AudioOut>; 2],
    processor: olivia_core::processor::Processor,
}

impl Processor {
    pub fn new(
        client: &jack::Client,
        processor: olivia_core::processor::Processor,
    ) -> Result<Processor, jack::Error> {
        let midi_input = client.register_port("midi_input", jack::MidiIn::default())?;
        let outputs = [
            client.register_port("output_l", jack::AudioOut::default())?,
            client.register_port("output_r", jack::AudioOut::default())?,
        ];
        Ok(Processor {
            midi_input,
            outputs,
            processor,
        })
    }
}

impl jack::ProcessHandler for Processor {
    fn process(&mut self, _: &jack::Client, ps: &jack::ProcessScope) -> jack::Control {
        let mut ports = match &mut self.outputs {
            [left, right] => olivia_core::processor::Ports {
                left: left.as_mut_slice(ps),
                right: right.as_mut_slice(ps),
            },
        };
        for _ in self.midi_input.iter(ps) {}
        self.processor.process(&mut ports);
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
