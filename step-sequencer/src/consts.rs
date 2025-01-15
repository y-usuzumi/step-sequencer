use std::time::Duration;

use crate::{drum_track::Beat, midi::note::Note};

pub const TRACK_DEFAULT_BEAT: Beat = Beat {
    channel: 9,
    note: Note::C(1),
    velocity: 72,
};
pub const BEAT_TIME_MICRO: u32 = 1_000_000;

pub const TIMELINE_TICK_DURATION: Duration = Duration::from_millis(10);
