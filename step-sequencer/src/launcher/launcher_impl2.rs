use std::sync::{Arc, RwLock};

use crate::{
    audio::{create_ss_client2, SSClient2},
    beatmaker::beat_sorter::{self, BeatSorter},
    project::Project,
    SSResult,
};

use super::SSLauncher;

pub struct SSLauncherImpl2 {
    project: Project,
    ss_client2: Box<dyn SSClient2 + Send>,
}

impl SSLauncherImpl2 {
    pub fn new() -> Self {
        let project = Project::new();
        let beat_sorter = BeatSorter::with_tracks(project.tracks());
        let ss_client2 = create_ss_client2(
            Arc::new(RwLock::new(beat_sorter)),
            project.project_settings(),
        );
        Self {
            project,
            ss_client2,
        }
    }

    pub fn project(&self) -> &Project {
        &self.project
    }
}

impl SSLauncher for SSLauncherImpl2 {
    fn start(&mut self) -> SSResult<()> {
        self.ss_client2.start()
    }

    fn pause(&mut self) -> SSResult<()> {
        self.ss_client2.pause()
    }

    fn stop(&mut self) -> SSResult<()> {
        self.ss_client2.stop()
    }

    fn project(&self) -> &Project {
        &self.project
    }

    fn subscribe_to_beats(&self) -> crate::beatmaker::BeatMakerSubscription {
        unimplemented!("TODO")
    }

    fn send_command(&self, command: crate::command::Command) -> SSResult<()> {
        unimplemented!("TODO")
    }
}
