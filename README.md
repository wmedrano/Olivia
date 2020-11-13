# Olivia

Olivia will be a tool for composing music.

# Directories

## Backend

Rust binary that provides all the processing functionality for Olivia. It can
be controlled through HTTP calls.

## Core

Rust library that provides barebones functionalities for generating audio.

## Devtools

Collection of tools that will aide in development.

## Lilv

Rust wrapper for Lilv, a C library for discovering and instantiating LV2 audio
plugins. This wrapper is a fork of https://github.com/grejppi/lilv-rs.

# Requirements

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
