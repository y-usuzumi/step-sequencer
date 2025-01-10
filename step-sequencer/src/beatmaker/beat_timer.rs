use std::sync::{Arc, RwLock};

use derive_builder::Builder;
use log::{debug, info};

use crate::{
    project::{ProjectSettings, F},
    timeline::{TimelineEvent, TimelineSubscription},
};

#[deprecated = "In favor of BeatSorter. May revive if later determined that per-track BeatTimer is a better option."]
#[derive(Builder)]
#[builder(pattern = "owned")]
pub struct BeatTimer {
    timeline_subscription: TimelineSubscription,
    project_settings: Arc<RwLock<ProjectSettings>>,
}

fn bpm_to_millis(bpm: u16) -> F {
    F::from(60_000) / F::from(bpm)
}

impl BeatTimer {
    pub fn run_forever(&self, on_beat: impl Fn(usize), on_pause: impl Fn(), on_stop: impl Fn()) {
        let interval = self.timeline_subscription.interval;
        let (mut current_beat, mut current_beat_frac) = (0usize, F::from(0));
        for tick in self.timeline_subscription.receiver.iter() {
            match tick {
                TimelineEvent::Tick(tick) => {
                    if tick == 0 {
                        on_beat(0);
                        continue;
                    }
                    let beat_interval = bpm_to_millis(self.project_settings.read().unwrap().tempo);
                    current_beat_frac += F::from(interval.as_millis()) / beat_interval;
                    while current_beat_frac > F::from(1) {
                        (current_beat, current_beat_frac) =
                            (current_beat + 1, current_beat_frac - F::from(1));
                        on_beat(current_beat);
                    }
                    *self
                        .project_settings
                        .read()
                        .unwrap()
                        .current_beat
                        .write()
                        .unwrap() = (current_beat, current_beat_frac);
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
                        .unwrap() = (0, F::from(0));
                    current_beat = 0;
                    on_stop();
                }
            }
        }
    }
}
