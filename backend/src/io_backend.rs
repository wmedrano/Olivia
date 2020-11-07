pub trait IoBackend {
    fn name(&self) -> &'static str;
    fn buffer_size(&self) -> usize;
    fn sample_rate(&self) -> f32;
    fn run_process_loop(self);
}
