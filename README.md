# midi-strip-client

A client meant to be used with [led-strip-server](https://github.com/Prior99/led-strip-server).

It processes MIDI input and converts it to RGB color by using the note and velocity.

The color will then be sent to the LED strip live.

## Usage

List all MIDI devices and use their IDs to select the MIDI device to listen on.

```
midi-strip-client midi-devices
midi-strip-client start --url ws://url-to-server --midi 1
```

## Building

Build the project as usual with cargo:

```
cargo build --release
```

## Contributors

 - Frederick Gnodtke
