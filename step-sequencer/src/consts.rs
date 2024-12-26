use crate::{drum_track::Beat, midi::note::Note};

pub const TRACK_DEFAULT_BEAT: Beat = Beat {
    channel: 9,
    note: Note::C(1),
    velocity: 72,
};
