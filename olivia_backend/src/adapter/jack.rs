#[derive(Debug)]
pub struct Processor {
    midi_input: jack::Port<jack::MidiIn>,
    outputs: [jack::Port<jack::AudioOut>; 2],
}

impl Processor {
    pub fn new(client: &jack::Client) -> Result<Processor, jack::Error> {
        let midi_input = client.register_port("midi_input", jack::MidiIn::default())?;
        let outputs = [
            client.register_port("output_l", jack::AudioOut::default())?,
            client.register_port("output_r", jack::AudioOut::default())?,
        ];
        Ok(Processor{
            midi_input,
            outputs,
        })
    }
}

impl jack::ProcessHandler for Processor {
    fn process(&mut self, _: &jack::Client, ps: &jack::ProcessScope) -> jack::Control {
        for output in self.outputs.iter_mut() {
            for o in output.as_mut_slice(ps).iter_mut() {
                *o = 0.0;
            }
        }
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
