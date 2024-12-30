Beatmaker in Yukio's Step Sequencer
===

The timeline also runs in its own thread. It makes the best attempt to ensure the interval of each recurring execution is as close to the given interval (10ms by default).

The timeline uses the same pub/sub model as the BeatMaker, although the channel it sends ticks to is a sync channel. This means if the subscribers consume the ticks slower than the speed of send, the ticks will fill up the channel and block the timeline. In this case, the next time the timeline attempts to send a tick, it evaluates how many ticks were lagged behind and skip them.