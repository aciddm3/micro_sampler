# Microsampler

Microsampler is a simple polyphonic sampler capable of playing back audio files in .wav, .mp3, .flac, and .ogg formats.

## Specifications

Microsampler always plays a single channel from the loaded file. The channel is selected by its index using the "+" and "-" keys in the user interface.

It features adjustable playback range and looping settings.

Polyphony can be configured within the range of 1 to MAX_POLYPHONY. The MAX_POLYPHONY constant can be set to alternative values before compilation.

It includes controls for output volume, panning, and transposition.

The sampler features an ADSR envelope that controls the amplitude.

Looping and retriggering can be enabled or disabled via separate toggles.

## Crates Used

An excerpt from the cargo.toml file:

```toml
itertools = "0.15.0"
nih_plug = { git = "[https://github.com/robbert-vdh/nih-plug](https://github.com/robbert-vdh/nih-plug)", features = ["vst3"] }
nih_plug_derive = { git = "[https://github.com/robbert-vdh/nih-plug](https://github.com/robbert-vdh/nih-plug)" }
nih_plug_egui = { git = "[https://github.com/robbert-vdh/nih-plug.git](https://github.com/robbert-vdh/nih-plug.git)" }
parking_lot = "0.12.5"
rfd = "0.17.2"
symphonia = { version = "0.5", features = ["mp3", "flac", "ogg", "vorbis", "wav", "pcm"] }
```
The project also includes an integrated xtask for quick compilation of the vst3-bundle:

```sh
cargo run --package xtask bundle micro_sampler --release
```
