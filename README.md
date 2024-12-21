## Yukio's Step Sequencer

The goal is to create a Step Sequencer that supports both JACK (Linux) and CoreAudio (macOS).

Main milestones
- [ ] Can run either as a standalone app or a VST/LV2/CLAP plugin
- [ ] Supports both MIDI/Audio output
- [ ] Supports keyboard/mouse control
- [ ] Supports MIDI in as controller
- [ ] Steps, substeps
- [ ] Different channels
...etc


## Current status

I'm currently still learning the basic concepts of JACK, CoreAudio and even step sequencer itself. I've ordered a Yamaha Seqtrak which
should help me understand what features are needed.

Code-wise, I've currently only implemented basic MIDI event data structure and some simple test examples.
On macOS, it is able to produce sine wave audio through CoreAudio.
On Linux, it is able to produce sine wave audio through JACK.
On Linux, it is able to produce alternating series of Note on/off events. It can be configured to route to Bitwig using a2jmidid.
