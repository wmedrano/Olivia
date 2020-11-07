pub trait IoBackend {
    fn name(&self) -> &'static str;
    fn buffer_size(&self) -> usize;
    fn run_process_loop(self);
}
