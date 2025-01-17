## MIDI libraries

There are a few MIDI libraries out there, including low-level C API bindings, high-level abstractions of a single audio framework and platform-independent libraries.

### [RustAudio/rust-jack](https://github.com/RustAudio/rust-jack)

Rust bindings to JACK API. The repo itself also contains jack-sys which is a low-level thin wrapper around C API. This is what is currently used by step-sequencer to enable JACK support.

#### Pros

* Faithful Rust implementation without any (that I found) unsafe Rust.
* Callback based, which adheres to the C API. This enables precise time processing.

#### Cons

### [chris-zen/coremidi](https://github.com/chris-zen/coremidi)

Rust bindings to CoreMIDI API, based on [jonas-k/coremidi-sys](https://github.com/jonas-k/coremidi-sys).

#### Pros

* Getting the job done is simple and easy (essentially by simply calling the send method)

#### Cons

* Hard to implement precise timing. It accepts a timestamp that represents the time on host to play the note, which means it is possible to send a lot of notes to play in advance.
  However CoreMIDI does not provide any API to get the current buffer size (which according to AI is dynamic). If we send millions of notes it will definitely cause buffer overflow, so we
  have to send notes at controlled speed.


### [Boddlnagg/midir](https://github.com/Boddlnagg/midir)

Cross-platform, realtime MIDI processing in Rust. It supports every platform that step-sequencer supports / plans to support.

#### Pros

* Cross-platform
* By reading the code, it seems all MIDI input is handled in callback, and all MIDI output is handled in a synchronous manner, similar to CoreMIDI.

#### Cons

* **DEALBREAKER** It [does not even accept a timestamp in the send method](https://github.com/Boddlnagg/midir/blob/master/src/backend/jack/mod.rs#L405). In its JACK implementation, it maintains a ringbuffer and always pushes your message at the ringbuffer write pointer i.e. you have no control over the exact time of the message. In its CoreMIDI implementation, it always uses
the [host timestamp or 0](https://github.com/Boddlnagg/midir/blob/master/src/backend/coremidi/mod.rs#L407-L411) (which according to [CoreMIDI documentation](https://developer.apple.com/documentation/coremidi/midipacket/1495113-timestamp)) does exactly the same thing.
