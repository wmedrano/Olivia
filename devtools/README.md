# Devtools

Devtools contains a bunch of utilities to ease developing for Olivia.

## Dev Setup

dev_setup provides a JACK endpoints for test instances of audio and midi
devices. If you have issues with JACK IO, try setting JACK's backend as Dummy
and using dev_setup to output audio. Note: there may be noticeably more latency
but this should be OK for development purposes.

### Requirements

- Must have Cargo and the Rust compiler installed. See your package manager or
    rustup.rs for details.
- Must have JACK development libraries installed.
- Must have SDL2 development libraries installed.

### Running

Before running, start the JACK server. This can be done using qjackctl or
through command line with `jackd -r -ddummy -r44100 -p1024`.

```bash
# Start JACK server manually as dev_setup does not yet do this.

# Run dev_setup
cd dev_setup
cargo run --release
# You should now see olivia_dev devices in the JACK graph.
```

## SDL2 Development Libraries

SDL2 is a crossplatform library meant for developing games. They support audio,
inputs (keyboard, mice, touch, gamepads), displays and a bunch of other stuff.
SDL2 can be installed through your package manager.
