use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

use derive_builder::Builder;
use log::{debug, info};

use crate::{
    project::ProjectSettings,
    timeline::{TimelineEvent, TimelineSubscription},
};

#[derive(Builder)]
#[builder(pattern = "owned")]
pub struct BeatTimer {
    timeline_subscription: TimelineSubscription,
    project_settings: Arc<RwLock<ProjectSettings>>,
}

fn bpm_to_duration(bpm: u16) -> Duration {
    Duration::from_secs_f64(60. / (bpm as f64))
}

impl BeatTimer {
    pub fn run_forever<T>(&self, on_beat: T)
    where
        T: Fn(u64),
    {
        // This method might pose a problem when the old tempo is very low, since
        // the tempo change needs to wait until the current `thread::sleep` is done
        // ... maybe a per-millisecond tick is better?

        let mut next_beat_time = 0;
        let interval = self.timeline_subscription.interval;
        let mut current_beat = 0;
        for tick in self.timeline_subscription.receiver.iter() {
            match tick {
                TimelineEvent::Tick(tick) => {
                    while next_beat_time <= interval.as_millis() * (tick as u128) {
                        on_beat(current_beat);
                        let beat_interval =
                            bpm_to_duration(self.project_settings.read().unwrap().tempo)
                                .as_millis();
                        *self
                            .project_settings
                            .read()
                            .unwrap()
                            .current_beats
                            .write()
                            .unwrap() = current_beat;
                        next_beat_time += beat_interval;
                        current_beat += 1;
                    }
                }
                TimelineEvent::Pause => {}
                TimelineEvent::Stop => {
                    current_beat = 0;
                    next_beat_time = 0;
                }
            }
        }
    }
}
