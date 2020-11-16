// Name of the JACK client that will playback audio.
pub const CLIENT_NAME: &'static str = "olivia_dev_playback";

// Name of input ports that will be rerouted to output audio to the default
// playback device.
pub const PLAYBACK_PORTS: [&'static str; 2] = ["playback_1", "playback_2"];

// Wraps an sdl2 audio queue with Send support.
enum AudioQueueWrapper {
    Sdl(sdl2::audio::AudioQueue<f32>),
    Dummy,
}

impl AudioQueueWrapper {
    fn new(q: sdl2::audio::AudioQueue<f32>) -> AudioQueueWrapper {
        q.resume();
        AudioQueueWrapper::Sdl(q)
    }

    fn dummy() -> AudioQueueWrapper {
        AudioQueueWrapper::Dummy
    }

    fn queue(&mut self, audio: &[f32]) -> bool {
        match self {
            AudioQueueWrapper::Sdl(q) => q.queue(audio),
            AudioQueueWrapper::Dummy => true,
        }
    }
}

// We implement Send to use the audio queue in the process Jack thread.
// This is technically unsafe since JACK requires a callback to have a static
// lifetime and it is possible for JACK to outlive SDL2 in cases of errors.
// TODO: find a way to implement this without unsafe.
unsafe impl Send for AudioQueueWrapper {}

pub fn run() {
    let (client, status) =
        jack::Client::new(CLIENT_NAME, jack::ClientOptions::NO_START_SERVER).unwrap();
    println!(
        "Started olivia_dev client {} with status {:?}.",
        client.name(),
        status
    );

    let sdl_context = sdl2::init().unwrap();
    let sdl_audio = sdl_context.audio().unwrap();
    let spec = sdl2::audio::AudioSpecDesired {
        freq: Some(client.sample_rate() as i32),
        channels: Some(2),
        samples: Some(client.buffer_size() as u16),
    };
    let mut queue = match sdl_audio.open_queue(None, &spec) {
        Ok(q) => AudioQueueWrapper::new(q),
        Err(e) => {
            println!("Could not open audio device with sdl2: {:?}", e);
            println!("Playback will not be supported.");
            AudioQueueWrapper::dummy()
        }
    };

    let inputs: Vec<_> = PLAYBACK_PORTS
        .iter()
        .map(|n| client.register_port(n, jack::AudioIn::default()).unwrap())
        .collect();
    for i in inputs.iter() {
        println!(
            "Registered audio output port {}.",
            i.name().unwrap_or("ERROR_GETTING_PORT_NAME".to_string())
        );
    }
    let mut temp_buffer: Vec<f32> = Vec::with_capacity(2 * 44100);
    let process = jack::ClosureProcessHandler::new(
        move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            let in_l = inputs[0].as_slice(ps);
            let in_r = inputs[1].as_slice(ps);
            temp_buffer.clear();
            for (l, r) in in_l.iter().zip(in_r.iter()) {
                temp_buffer.push(*l);
                temp_buffer.push(*r);
            }
            if !queue.queue(&temp_buffer) {
                println!("Could not write to SDL2 audio output. Writing operation will cease.");
                jack::Control::Quit
            } else {
                jack::Control::Continue
            }
        },
    );

    let active_client = client.activate_async((), process).unwrap();
    std::thread::park();
    active_client.deactivate().unwrap();
}
