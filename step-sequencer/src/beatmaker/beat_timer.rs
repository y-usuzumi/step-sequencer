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
    pub fn run_forever(&self, on_beat: impl Fn(u64), on_pause: impl Fn(), on_stop: impl Fn()) {
        // This method might pose a problem when the old tempo is very low, since
        // the tempo change needs to wait until the current `thread::sleep` is done
        // ... maybe a per-millisecond tick is better?

        let interval = self.timeline_subscription.interval;
        let (mut current_beat, mut current_beat_micros) = (0u64, 0u32);
        for tick in self.timeline_subscription.receiver.iter() {
            match tick {
                TimelineEvent::Tick(tick) => {
                    if tick == 0 {
                        on_beat(0);
                        continue;
                    }
                    let beat_interval =
                        bpm_to_duration(self.project_settings.read().unwrap().tempo).as_millis();
                    current_beat_micros +=
                        (interval.as_millis() as u32) * 1_000_000 / (beat_interval as u32);
                    while current_beat_micros >= 1_000_000 {
                        (current_beat, current_beat_micros) =
                            (current_beat + 1, current_beat_micros - 1_000_000);
                        on_beat(current_beat);
                    }
                    *self
                        .project_settings
                        .read()
                        .unwrap()
                        .current_beat
                        .write()
                        .unwrap() = (current_beat, current_beat_micros);
                }
                TimelineEvent::Pause => {
                    on_pause();
                }
                TimelineEvent::Stop => {
                    *self
                        .project_settings
                        .read()
                        .unwrap()
                        .current_beat
                        .write()
                        .unwrap() = (0, 0);
                    current_beat = 0;
                    on_stop();
                }
            }
        }
    }
}
