use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

use crate::{
    drum_track::{Beat, DrumTrack},
    id::SSId,
    midi::ChannelVoiceEvent,
    project::{TrackMap, F},
};

use super::beat_time::BeatTime;

pub struct BeatSorter {
    tracks: Arc<RwLock<TrackMap>>,
    current_beat_time: BeatTime,
}

fn find_next_beat_in_track(track: &DrumTrack, beat_time: BeatTime) -> usize {
    let idx = if beat_time.fraction() > F::from(0) {
        // We are between two beats
        beat_time.integral() + 1
    } else {
        beat_time.integral()
    };
    return idx % track.len();
}

/// Get beats from start (inclusive) to end (exclusive) respecting track tempo scale
/// `start` and `end` are global beat time (i.e. when tempo scale = 1)
fn get_beats_between_in_track(
    track: &DrumTrack,
    start: BeatTime,
    end: BeatTime,
) -> Vec<(BeatTime, Beat)> {
    let tempo_scale = track.get_tempo_scale();
    let mut start_scaled = start.stretch(tempo_scale).ceil();
    let end_scaled = end.stretch(tempo_scale);
    let mut result = vec![];
    while start_scaled < end_scaled {
        let idx = find_next_beat_in_track(track, start_scaled);
        let beats = track.get_as_beats(idx).flatten();
        if let Some(beats) = beats {
            result.extend(
                beats
                    .into_iter()
                    .map(|b| (start_scaled.compress(tempo_scale), b)),
            );
        }
        start_scaled = start_scaled.add_integral(1);
    }

    return result;
}

fn beat_to_channel_voice_events(beat: Beat) -> [ChannelVoiceEvent; 2] {
    [
        ChannelVoiceEvent::NoteOn {
            channel: beat.channel,
            key: beat.note.into(),
            velocity: beat.velocity,
        },
        ChannelVoiceEvent::NoteOff {
            channel: beat.channel,
            key: beat.note.into(),
            velocity: beat.velocity,
        },
    ]
}

impl BeatSorter {
    pub fn with_tracks(tracks: Arc<RwLock<TrackMap>>) -> Self {
        Self {
            tracks,
            current_beat_time: BeatTime::zero(),
        }
    }

    pub fn reset(&mut self) {
        self.current_beat_time = BeatTime::zero();
    }

    pub fn jump(&mut self, beat_time: BeatTime) {
        self.current_beat_time = beat_time;
    }

    pub fn advance(
        &mut self,
        next_beat_time: BeatTime,
    ) -> Vec<(BeatTime, Vec<(SSId, ChannelVoiceEvent)>)> {
        let mut treemap: BTreeMap<BeatTime, Vec<_>> = BTreeMap::new();
        for (id, track) in self.tracks.read().unwrap().iter() {
            for (beat_time, beat) in
                get_beats_between_in_track(track, self.current_beat_time, next_beat_time)
            {
                let [note_on, note_off] = beat_to_channel_voice_events(beat);
                treemap.entry(beat_time).or_default().push((*id, note_on));
                treemap
                    .entry(beat_time.add_fraction(F::new(1u64, 4u64)))
                    .or_default()
                    .push((*id, note_off));
            }
        }
        self.current_beat_time = next_beat_time;
        return treemap.into_iter().collect();
    }
}
