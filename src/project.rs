use std::sync::{Arc, RwLock};

use crate::{beatmaker::BeatMaker, drum_track::DrumTrack, SSResult};

pub struct Project<'a> {
    tracks: Arc<RwLock<Vec<DrumTrack>>>,
    project_settings: Arc<RwLock<ProjectSettings>>,
    beatmaker: &'a BeatMaker,
}

pub struct ProjectSettings {
    pub tempo: u16,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self { tempo: 110 }
    }
}

impl<'a> Project<'a> {
    pub fn new(beatmaker: &'a BeatMaker) -> Self {
        Self {
            tracks: Arc::new(RwLock::new(Vec::new())),
            project_settings: Arc::new(RwLock::new(ProjectSettings::default())),
            beatmaker: beatmaker,
        }
    }

    pub fn add_track(&self) -> usize {
        let mut tracks = self.tracks.write().unwrap();
        tracks.push(DrumTrack::new());
        tracks.len() - 1
    }

    pub fn play(&mut self) {
        self.beatmaker.start(&self);
    }

    pub fn project_settings(&self) -> Arc<RwLock<ProjectSettings>> {
        self.project_settings.clone()
    }

    pub fn tracks(&self) -> Arc<RwLock<Vec<DrumTrack>>> {
        self.tracks.clone()
    }
}
