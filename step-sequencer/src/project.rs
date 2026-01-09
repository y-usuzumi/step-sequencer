use std::sync::{Arc, RwLock};

use fraction::Fraction;
use indexmap::IndexMap;

use crate::{
    beatmaker::beat_time::BeatTime,
    drum_track::DrumTrack,
    id::{new_id, SSId},
};

pub type F = Fraction;

pub type TrackMap = IndexMap<SSId, DrumTrack>;
pub type Tempo = u16;
pub type TempoScale = F;

pub struct Project {
    tracks: Arc<RwLock<TrackMap>>,
    project_settings: Arc<RwLock<ProjectSettings>>,
}

pub struct ProjectSettings {
    pub tempo: Tempo,
    pub current_beat_time: Arc<RwLock<BeatTime>>,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            tempo: 220,
            current_beat_time: Arc::new(RwLock::new(BeatTime::zero())),
        }
    }
}

impl Project {
    pub fn new() -> Self {
        Self {
            tracks: Arc::new(RwLock::new(IndexMap::new())),
            project_settings: Arc::new(RwLock::new(ProjectSettings::default())),
        }
    }

    pub fn add_track(&self, track: DrumTrack) -> SSId {
        let mut tracks = self.tracks.write().unwrap();
        let track_id = new_id();
        tracks.insert(track_id, track);
        track_id
    }

    pub fn add_empty_track(&self) -> SSId {
        self.add_track(DrumTrack::new("Unnamed"))
    }

    pub fn remove_track(&self, track_id: SSId) -> Option<DrumTrack> {
        let mut tracks = self.tracks.write().unwrap();
        tracks.shift_remove(&track_id)
    }

    pub fn project_settings(&self) -> Arc<RwLock<ProjectSettings>> {
        self.project_settings.clone()
    }

    pub fn tracks(&self) -> Arc<RwLock<TrackMap>> {
        self.tracks.clone()
    }
}
