Beatmaker in Yukio's Step Sequencer
===

The beatmaker runs on its own thread. It implements a Pub/Sub model via the use of mpsc for each subscriber.

I have thought about other design choices about beatmaker.

For example, a naive approach would be a callback mechanism, in which the user passes a callback function that gets called upon every beat.
The problem is that in JACK framework, your main process code is also a callback function which is called on every process cycle.
You don't get to do something at any time of your discretion.

With the Pub/Sub model, I can get a receiver channel of beatmaker's MIDI events, which I can drain on every process cycle.

TODO: Assign a timestamp to every midi event sent from BeatMaker, so that I can put the message aside if I decide it's too early to send it
to the output port.
