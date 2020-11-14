mod midi;
mod playback;

fn main() {
    std::thread::spawn(|| playback::run());
    std::thread::sleep(std::time::Duration::from_secs(1));

    midi::run();
}
