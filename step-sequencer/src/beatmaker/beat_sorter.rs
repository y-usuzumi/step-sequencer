use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

use crate::{
    drum_track::Beat,
    project::{BeatTime, TrackMap},
};

use crate::project::F;

#[derive(Default)]
pub struct BeatSorter {
    treemap: BTreeMap<BeatTime, Vec<Option<Beat>>>,
    stored_next_beat: Option<usize>,
}

impl BeatSorter {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn len(&self) -> usize {
        self.treemap.len()
    }

    pub fn stored_next_beat(&self) -> Option<usize> {
        self.stored_next_beat
    }

    pub fn set_stored_next_beat(&mut self, next_beat: usize) {
        self.stored_next_beat = Some(next_beat);
    }

    pub fn next_beat_time(&self) -> Option<BeatTime> {
        self.treemap.first_key_value().map(|(k, _)| *k)
    }

    pub fn reset(&mut self) {
        self.treemap.clear();
        self.stored_next_beat = None;
    }

    /// Given drum tracks, push the beats in each track that should
    /// make sound before (and not include) the next *global* beat
    /// For example, given the following 3 tracks:
    ///
    /// T1: A-------------------B-------------------C  x1
    /// T2: D-------E-------F-------G-------H-------I  x5/2
    /// T3: J-----------------------------K-----------------------------L x2/3
    ///
    /// T1 has a tempo scale of x1, which means it follows global tempo
    /// T2 has a tempo scale of x2.5, which means it is 2.5 times the speed of global tempo
    /// T3 has a tempo scale of x0.66, which means it is one third the speed of global tempo
    ///
    /// If we call push(0, &tracks), we are storing all beats from A(incl) to B(excl),
    /// namely A, D, E, F, J.
    /// Similarly, if we call push(1, &tracks), we are storing all beats from B(incl) to C(excl),
    /// namely B, G, H, K.
    pub fn push(&mut self, beat_seq_num: usize, tracks: &TrackMap) {
        for (_, track) in tracks.iter() {
            let tempo_scale = track.get_tempo_scale();
            if tempo_scale == F::from(1) {
                // No scaling
                let beat_idx = beat_seq_num % track.len();
                self.treemap
                    .entry((beat_seq_num, F::from(0)))
                    .or_default()
                    .push(track.get_as_beat(beat_idx).flatten());
            } else if tempo_scale > F::from(1) {
                // Faster
                let beat_idx =
                    usize::try_from((F::from(beat_seq_num) * tempo_scale).ceil()).unwrap();
                let beat_idx_after_next_unscaled_beat =
                    usize::try_from((F::from(beat_seq_num + 1) * tempo_scale).ceil()).unwrap();
                for idx in beat_idx..beat_idx_after_next_unscaled_beat {
                    let beat_time = F::from(idx) / tempo_scale;
                    let beat_time: BeatTime = (
                        usize::try_from(beat_time.trunc()).unwrap(),
                        beat_time.fract(),
                    );
                    self.treemap
                        .entry(beat_time)
                        .or_default()
                        .push(track.get_as_beat(idx % track.len()).flatten());
                }
            } else {
                // Slower
                let next_nearest_scaled_beat = (F::from(beat_seq_num + 1) * tempo_scale).floor();
                println!("NEXT: {}", next_nearest_scaled_beat);
                if (F::from(beat_seq_num) * tempo_scale).ceil() == next_nearest_scaled_beat {
                    let beat_time = next_nearest_scaled_beat / tempo_scale;
                    let beat_time: BeatTime = (
                        usize::try_from(beat_time.trunc()).unwrap(),
                        beat_time.fract(),
                    );
                    self.treemap.entry(beat_time).or_default().push(
                        track
                            .get_as_beat(
                                usize::try_from(next_nearest_scaled_beat).unwrap() % track.len(),
                            )
                            .flatten(),
                    );
                }
            }
        }
    }

    /// Pop the next earliest beat to sound.
    /// For example, given the following 3 tracks:
    ///
    /// T1: A-------------------B-------------------C  x1
    /// T2: D-------E-------F-------G-------H-------I  x5/2
    /// T3: J-----------------------------K-----------------------------L x2/3
    ///
    /// T1 has a tempo scale of x1, which means it follows global tempo
    /// T2 has a tempo scale of x2.5, which means it is 2.5 times the speed of global tempo
    /// T3 has a tempo scale of x0.66, which means it is one third the speed of global tempo
    ///
    /// If we have called push(0, &tracks), we should have A, D, E, F, J in store.
    /// As we pop, we will get:
    ///   [A, D, J] (earliest and they share the same beat time),
    ///   [E]
    ///   [F]
    pub fn pop(&mut self) -> Option<(BeatTime, Vec<Option<Beat>>)> {
        self.treemap.pop_first()
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::{
        consts::TRACK_DEFAULT_BEAT,
        drum_track::{Beat, DrumTrack},
        midi::note::Note,
        project::TrackMap,
        project::F,
    };

    use super::BeatSorter;

    macro_rules! test_beat {
        ($note:ident) => {
            Some(Beat {
                note: Note::$note(1),
                ..TRACK_DEFAULT_BEAT
            })
        };
    }

    #[test]
    fn test_beat_sorter_1() {
        let mut tracks = TrackMap::new();
        let track_ids: Vec<_> = (0..3).map(|_| Uuid::new_v4()).collect();
        println!("{:?}", track_ids);
        tracks.insert(
            track_ids[0],
            DrumTrack::with_beats_for_test(F::from(1), Note::C(1), 3),
        );
        tracks.insert(
            track_ids[1],
            DrumTrack::with_beats_for_test(F::new(5u64, 2u64), Note::D(1), 3),
        );
        tracks.insert(
            track_ids[2],
            DrumTrack::with_beats_for_test(F::new(2u64, 3u64), Note::E(1), 3),
        );

        let mut beat_sorter = BeatSorter::new();
        {
            // First global beat: first beats of all tracks
            beat_sorter.push(0, &tracks);
            assert_eq!(beat_sorter.len(), 3);
            assert_eq!(
                beat_sorter.pop().unwrap(),
                (
                    (0, F::from(0)),
                    vec![test_beat!(C), test_beat!(D), test_beat!(E)]
                )
            );
            assert_eq!(
                beat_sorter.pop().unwrap(),
                ((0, F::new(2u64, 5u64)), vec![test_beat!(D)])
            );
            assert_eq!(
                beat_sorter.pop().unwrap(),
                ((0, F::new(4u64, 5u64)), vec![test_beat!(D)])
            );
        }
        {
            // Second global beat
            beat_sorter.push(1, &tracks);
            // assert_eq!(beat_sorter.len(), 4);
            assert_eq!(
                beat_sorter.pop().unwrap(),
                ((1, F::from(0)), vec![test_beat!(C)])
            );
            assert_eq!(
                beat_sorter.pop().unwrap(),
                ((1, F::new(1u64, 5u64)), vec![test_beat!(D)])
            );
            assert_eq!(
                beat_sorter.pop().unwrap(),
                ((1, F::new(1u64, 2u64)), vec![test_beat!(E)])
            );
            assert_eq!(
                beat_sorter.pop().unwrap(),
                ((1, F::new(3u64, 5u64)), vec![test_beat!(D)])
            );
        }
    }
}
