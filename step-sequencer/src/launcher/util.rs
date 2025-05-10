use std::sync::RwLockWriteGuard;

use crate::{drum_track::DrumTrack, project::TrackMap};

pub(super) fn get_track<'a>(
    track_map: &'a mut RwLockWriteGuard<TrackMap>,
    track_idx: usize,
) -> Option<&'a mut DrumTrack> {
    let mut tracks = track_map.values_mut();
    for _ in 0..track_idx {
        tracks.next();
    }
    tracks.next()
}
