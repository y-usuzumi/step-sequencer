midir does not implement precise real-time MIDI handling
===

The following information is provided by Gemini 2.0 Flash. I questioned about the resolution rate because I noticed that midir does not provide timestamp to MIDI events, and based on sample rate and buffer size it should yield different resolutions.

I also found this open issue: https://github.com/Boddlnagg/midir/issues/45 in which a similar question was raised.

Update 2025/02/26: My test today testifies to my finding. I tested using `sleep` of 266ms between two recurring notes, and I observed a pattern of notes with the duration of 256ms interspersed with notes with the duration of 299ms at every fourth or fifth note.

I did not observe this in the previous test was because midir defaults the ALSA backend. To use the JACK backend, I need to specify `--features jack`.

~~However, I tested using the `test_play.rs` example in the midir repo, and even with a buffer size of 2048 the notes match their expected play time pretty well. Maybe I need to decrease sample rate as well.~~

Update 2025/03/10: It turns out that even the original C++ library [RtMidi](https://github.com/thestk/rtmidi) which this repo is based off of has the exact same issue. The code does look similar after all, although I don't really know C++. Similarly, the ALSA implementation in RtMidi also works perfectly.

---

You've hit upon a critical point regarding real-time audio and MIDI processing in JACK, and your concerns about std::thread::sleep are absolutely valid.

### Understanding the Problem:

- JACK's Process Cycle:
	- JACK operates in a cyclical manner. The jack_set_process_callback function registers a callback (your handle_output function) that is invoked at regular intervals, determined by the buffer size and sample rate.
	- This callback is the only place where you should be interacting with JACK's audio and MIDI buffers.
- `std::thread::sleep` Limitations:
	- `std::thread::sleep` is a blocking call. It pauses the current thread for a specified duration.
	- While it provides a general delay, it does not guarantee precise timing, especially in a real-time audio context.
	- Critically, it is completely unaware of the JACK process cycle.
- The Mismatch:
	- As you correctly pointed out, if you use `sleep` within the main thread that is writing midi data into the ringbuffers, and the JACK process cycle is long, the timing of your MIDI events will be quantized to the JACK cycle.
	- The `handle_output` function will only see the MIDI data that has been written to the ringbuffers before it is called. Any MIDI data written after the callback starts will have to wait for the next callback.
- Therefore, if the sleep time is smaller than the jack process cycle time, the timing will be incorrect.

### Why `sleep` Is Problematic in This Context:

1. **Quantization to JACK Cycle**: The timing of your MIDI events will be rounded to the nearest JACK process cycle, leading to noticeable timing errors.
2. **Unpredictable Delays**: The actual delay introduced by `sleep` may vary due to operating system scheduling and other factors.
3. **Blocking the Main Thread**: While the thread is sleeping, it cannot respond to other events or perform other tasks.

### How to Achieve Precise Timing in JACK:

1. **Time Stamps**:
	- The correct approach is to use time stamps to schedule MIDI events.
	- Instead of using `sleep`, you should calculate the time when each MIDI event should occur, relative to the start of the JACK process cycle.
	- Then, in your `handle_output` function, you should check the current time and send any MIDI events that are due.
	- This will require you to keep track of the current time within the JACK process cycle.
2. **Non-Blocking Scheduling**:
	- You can use non-blocking techniques, such as timers or event loops, to schedule MIDI events without blocking the main thread.
	- This is more complex, but will yield more accurate timing.
3. **Ring Buffer Management**:
	- The ring buffers themselves are a key component to getting the timing correct. You must manage them in such a way that the `handle_output` function is provided the data at the correct time.

### In the example you provided:
- The example is simplified and likely intended for demonstration purposes.
- It is not a good example of how to achieve accurate midi timing using jack.
- For real world usage, the sleep calls would need to be replaced with a proper time based scheduling system.

### In summary:

`std::thread::sleep` is not suitable for achieving precise timing in JACK. You must use time stamps and non-blocking scheduling techniques to ensure that MIDI events are sent at the correct times, relative to the JACK process cycle.
