use std::sync::Arc;

use crate::{
    audio::{create_ss_client2, SSClient2},
    beatmaker::beat_sorter::{self, BeatSorter},
    project::Project,
};

pub struct SSLauncher2 {
    beat_sorter: BeatSorter,
    project: Project,
    ss_client2: Box<dyn SSClient2 + Send>,
}

impl SSLauncher2 {
    pub fn new() -> Self {
        let project = Project::new();
        let beat_sorter = BeatSorter::with_tracks(project.tracks());
        let ss_client2 = create_ss_client2(Arc::new(beat_sorter));
        Self {
            beat_sorter,
            project,
            ss_client2,
        }
    }

    pub fn project(&self) -> &Project {
        &self.project
    }

    pub fn timeline(&self) -> &Timeline {
        &self.timeline
    }

    pub fn subscribe_to_beats(&self) -> BeatMakerSubscription {
        self.beatmaker.subscribe()
    }
}
