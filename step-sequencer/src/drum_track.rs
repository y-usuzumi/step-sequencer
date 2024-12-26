use self::DrumTrackBeat::*;
use crate::{
    consts,
    midi::{note::Note, Channel, Velocity},
};

#[derive(PartialEq, Clone, Debug)]
pub enum DrumTrackBeat {
    Unset,
    DefaultBeat,
    OverrideBeat(Beat),
}

pub struct DrumTrack {
    name: String,
    default_beat: Beat,
    beats: Vec<DrumTrackBeat>,
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct Beat {
    pub channel: Channel,
    pub note: Note,
    pub velocity: Velocity,
}

impl DrumTrack {
    pub fn new(name: &str) -> Self {
        Self::with_default_beat(name, consts::TRACK_DEFAULT_BEAT)
    }
    pub fn with_default_beat(name: &str, default_beat: Beat) -> Self {
        Self::with_beats(name, default_beat, &[])
    }

    pub fn with_beats(name: &str, default_beat: Beat, beats: &[DrumTrackBeat]) -> Self {
        Self {
            name: name.to_string(),
            default_beat,
            beats: beats.to_vec(),
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn get_default_beat(&self) -> Beat {
        self.default_beat
    }

    pub fn set_default_beat(&mut self, beat: Beat) {
        self.default_beat = beat;
    }

    pub fn toggle_beat(&mut self, idx: usize) {
        if let Unset = self.get(idx) {
            self.assign_beat(idx, DrumTrackBeat::DefaultBeat);
        } else {
            self.remove_beat(idx);
        }
    }

    pub fn assign_beat(&mut self, idx: usize, beat: DrumTrackBeat) {
        if self.beats.len() < idx + 1 {
            self.beats.resize(idx + 1, Unset);
        }

        self.beats[idx] = beat;
    }

    pub fn remove_beat(&mut self, idx: usize) {
        if let Some(v) = self.beats.get_mut(idx) {
            *v = Unset;
        }
    }

    pub fn resize(&mut self, size: usize) {
        self.beats.resize(size, Unset);
    }

    pub fn total_beats(&self) -> usize {
        self.beats.len()
    }

    pub fn get(&self, idx: usize) -> DrumTrackBeat {
        self.beats.get(idx).unwrap_or(&Unset).clone()
    }

    pub fn get_as_beat(&self, idx: usize) -> Option<Beat> {
        match self.get(idx) {
            Unset => None,
            DefaultBeat => Some(self.default_beat),
            OverrideBeat(beat) => Some(beat),
        }
    }
}
