use crate::io_backend::IoBackend;

pub struct DummyBackend(pub crate::controller::Processor);

impl IoBackend for DummyBackend {
    fn name(&self) -> &'static str {
        "Dummy"
    }
    fn buffer_size(&self) -> usize {
        4096
    }
    fn sample_rate(&self) -> f32 {
        44100.0
    }

    fn run_process_loop(self) {
        let mut s = self;
        let mut left = vec![0.0f32; s.buffer_size()];
        let mut right = vec![0.0f32; s.buffer_size()];
        loop {
            s.0.process(&[], &mut left, &mut right);
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}
