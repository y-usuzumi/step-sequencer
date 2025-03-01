## Yukio's Step Sequencer

A simple step sequencer that might one day evolve into a fully fledged DAW (Digital Audio Workstation).

## TUI support matrix

||Windows|Linux|macOS|
|--|--|--|--|
|JACK|✅[^1]|✅|❌|
|CoreAudio|🛑|🛑|✅|
|WinRT|❌|🛑|🛑|

✅: Supported
❌: Unsupported
🛑: Audio server not supported on platform

[^1]: JACK causes BSOD on my computer. The program itself works fine.
[^2]: Untested.

## GUI support matrix

TODO

## Releases

* v0.0.2: Tracker-style workspace. Tempo scale based on fraction numbers. New beat pattern. Do not start playing automatically on program start.
* v0.0.1: Rudimentary JACK and CoreAudio support. Plays fixed 4-track pattern that can be modified on the fly. Terminal UI.

## Project status

* [Kanban](https://github.com/users/y-usuzumi/projects/1/views/1)

## Design docs

* [Beatmaker](docs/Beatmaker.md)
* [Timeline](docs/Timeline.md)