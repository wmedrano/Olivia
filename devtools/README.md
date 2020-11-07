# Devtools

Devtools contains a bunch of utilities to ease developing for Olivia.

## JACK

JACK is a platform for realtime audio.

### Installation
For installation, see jackaudio.org/downloads/. For Linux systems, the package
manager is a suitable way for installing. For Mac and Windows, try downloading
and installing the precompiled versions; these should include the development
libraries as well.

### Basic Architecture

JACK starts a local server. The server is in charge of audio IO and routing
operations between clients. To start a server, the `jackd` command can be used.
For example, to start a server with a 1024 sample buffer size running at an
audio sample rate of 44.1kHz, run `jackd -r -ddummy -r44100 -p1024`. The other
way to start a server is using `qjackctl`. This may be installed using the
Linux package managers and/or downloading the precompiled JACK binaries.


## j2sdl2

j2sdl2 provides a JACK endpoint for outputting audio using the crossplatform
SDL2 audio libraries. If you have issues with JACK IO, try setting JACK's
backend as Dummy and using j2sdl2 to output audio. Note: there may be
noticeably more latency but this should be OK for development purposes.

### Requirements

- Must have Cargo and the Rust compiler installed. See your package manager or
    rustup.rs for details.
- Must have JACK development libraries installed.
- Must have SDL2 development libraries installed.

### Running

Before running, start the JACK server. This can be done using qjackctl or
through command line with `jackd -r -ddummy -r44100 -p1024`.

```bash
# Start JACK server.
cd j2sdl2
cargo run --release
# You should now see j2sdl2 in the JACK graph.
```

## SDL2 Development Libraries

SDL2 is a crossplatform library meant for developing games. They support audio,
inputs (keyboard, mice, touch, gamepads), displays and a bunch of other stuff.
SDL2 can be installed through your package manager.
