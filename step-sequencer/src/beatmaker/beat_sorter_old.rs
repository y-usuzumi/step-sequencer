use std::collections::BTreeMap;

use crate::{drum_track::Beat, id::SSId};

use super::BeatTime;

/// BeatSorter keeps track of the next note from each track to play
/// and ensure they are played in order.
#[derive(Default)]
pub struct BeatSorterOld {
    treemap: BTreeMap<BeatTime, Vec<(SSId, Option<Vec<Beat>>)>>,
}

impl BeatSorterOld {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn len(&self) -> usize {
        self.treemap.len()
    }

    pub fn reset(&mut self) {
        self.treemap.clear();
    }

    pub fn push(&mut self, track_id: SSId, beat_time: BeatTime, beat: Option<Vec<Beat>>) {
        self.treemap
            .entry(beat_time)
            .or_default()
            .push((track_id, beat));
    }

    pub fn pop(&mut self) -> Option<(BeatTime, Vec<(SSId, Option<Vec<Beat>>)>)> {
        self.treemap.pop_first()
    }

    pub fn top(&self) -> Option<(&BeatTime, &Vec<(SSId, Option<Vec<Beat>>)>)> {
        self.treemap.first_key_value()
    }
}
