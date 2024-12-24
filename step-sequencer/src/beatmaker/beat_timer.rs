use std::{
    sync::{Arc, RwLock},
    thread,
    time::{Duration, Instant},
};

use crate::project::ProjectSettings;

pub struct BeatTimer {
    project_settings: Arc<RwLock<ProjectSettings>>,
}

fn bpm_to_duration(bpm: u16) -> Duration {
    Duration::from_secs_f64(60. / (bpm as f64))
}

impl BeatTimer {
    pub fn with_project_settings(project_settings: Arc<RwLock<ProjectSettings>>) -> Self {
        Self {
            project_settings: project_settings,
        }
    }
    pub fn run_forever<T>(&self, on_beat: T)
    where
        T: Fn(u64),
    {
        // This method might pose a problem when the old tempo is very low, since
        // the tempo change needs to wait until the current `thread::sleep` is done
        // ... maybe a per-millisecond tick is better?
        let mut current_beat = 0;
        let mut next_time =
            Instant::now() + bpm_to_duration(self.project_settings.read().unwrap().tempo);
        loop {
            on_beat(current_beat);
            thread::sleep(next_time - Instant::now());
            next_time += bpm_to_duration(self.project_settings.read().unwrap().tempo);
            current_beat += 1;
        }
    }
}
