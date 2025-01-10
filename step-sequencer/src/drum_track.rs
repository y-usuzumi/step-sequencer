use self::DrumTrackBeat::*;
use crate::{
    consts,
    midi::{note::Note, Channel, Velocity},
    project::TempoScale,
};

use crate::project::F;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum DrumTrackBeat {
    Unset,
    DefaultBeat,
    OverrideBeat(Beat),
}

pub struct DrumTrack {
    name: String,
    tempo_scale: TempoScale,
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
            ..Default::default()
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn get_tempo_scale(&self) -> TempoScale {
        self.tempo_scale
    }

    pub fn get_default_beat(&self) -> Beat {
        self.default_beat
    }

    pub fn set_default_beat(&mut self, beat: Beat) {
        self.default_beat = beat;
    }

    pub fn set_default_channel(&mut self, channel: Channel) {
        self.default_beat = Beat {
            channel,
            ..self.get_default_beat()
        }
    }

    pub fn set_default_note(&mut self, note: Note) {
        self.default_beat = Beat {
            note,
            ..self.get_default_beat()
        }
    }

    pub fn set_default_velocity(&mut self, velocity: Velocity) {
        self.default_beat = Beat {
            velocity,
            ..self.get_default_beat()
        }
    }

    pub fn toggle_beat(&mut self, idx: usize) {
        if let Unset = self.beats.get(idx).unwrap_or(&Unset) {
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

    pub fn len(&self) -> usize {
        self.beats.len()
    }

    pub fn get(&self, idx: usize) -> Option<&DrumTrackBeat> {
        self.beats.get(idx)
    }

    pub fn get_as_beat(&self, idx: usize) -> Option<Option<Beat>> {
        self.beats.get(idx).map(|b| self.drum_track_beat_to_beat(b))
    }

    fn drum_track_beat_to_beat(&self, beat: &DrumTrackBeat) -> Option<Beat> {
        match beat {
            Unset => None,
            DefaultBeat => Some(self.default_beat),
            OverrideBeat(beat) => Some(*beat),
        }
    }

    pub fn iter(&self) -> std::slice::Iter<DrumTrackBeat> {
        self.beats.iter()
    }

    pub fn iter_as_beats(&self) -> impl Iterator<Item = Option<Beat>> + use<'_> {
        self.beats.iter().map(|b| self.drum_track_beat_to_beat(b))
    }

    #[cfg(test)]
    pub fn with_beats_for_test(tempo_scale: F, note: Note, count: usize) -> Self {
        use crate::consts::TRACK_DEFAULT_BEAT;

        Self {
            name: "who cares".to_string(),
            default_beat: Beat {
                note,
                ..TRACK_DEFAULT_BEAT
            },
            tempo_scale,
            beats: vec![DrumTrackBeat::DefaultBeat; count],
        }
    }
}

impl Default for DrumTrack {
    fn default() -> Self {
        Self {
            name: Default::default(),
            tempo_scale: F::from(1),
            default_beat: consts::TRACK_DEFAULT_BEAT,
            beats: Vec::new(),
        }
    }
}
