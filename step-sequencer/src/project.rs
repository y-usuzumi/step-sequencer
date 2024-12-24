use std::{collections::HashMap, sync::{Arc, RwLock}};

use crate::{drum_track::DrumTrack, id::{new_id, SSId}};

type TrackMap = HashMap<SSId, DrumTrack>;

pub struct Project {
    tracks: Arc<RwLock<TrackMap>>,
    project_settings: Arc<RwLock<ProjectSettings>>,
}

pub struct ProjectSettings {
    pub tempo: u16,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self { tempo: 110 }
    }
}

impl Project {
    pub fn new() -> Self {
        Self {
            tracks: Arc::new(RwLock::new(HashMap::new())),
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
        self.add_track(DrumTrack::new())
    }

    pub fn project_settings(&self) -> Arc<RwLock<ProjectSettings>> {
        self.project_settings.clone()
    }

    pub fn tracks(&self) -> Arc<RwLock<TrackMap>> {
        self.tracks.clone()
    }
}
