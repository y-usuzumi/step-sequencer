use std::sync::{Arc, RwLock};

use log::info;
use util::{beat_time_to_tick_and_offset, tick_to_beat_time};

use crate::beatmaker::beat_sorter::BeatSorter;
use crate::beatmaker::{BeatMakerSubscriberMap, BeatMakerSubscription, BeatMakerSubscriptionModel};
use crate::project::ProjectSettings;
use crate::SSResult;
pub mod adapter;
#[cfg(feature = "coreaudio")]
mod coreaudio;
#[cfg(feature = "jack")]
mod jack;
mod util;

struct AsyncProcessor {
    beat_sorter: Arc<RwLock<BeatSorter>>,
    project_settings: Arc<RwLock<ProjectSettings>>,
}

struct AsyncProcessorState {
    sample_rate: u64,
    buffer_size: u64,
    last_n_frames: u64,
    current_played_ticks: u64,
    beat_subscriber_map: Arc<RwLock<BeatMakerSubscriberMap>>,
}

impl AsyncProcessor {
    fn new(
        beat_sorter: Arc<RwLock<BeatSorter>>,
        project_settings: Arc<RwLock<ProjectSettings>>,
    ) -> Self {
        Self {
            beat_sorter,
            project_settings,
        }
    }
    fn on_process_cycle(
        &mut self,
        midi_adapter: &mut (dyn adapter::MIDIAdapter),
        state: &mut AsyncProcessorState,
        context: &crate::util::interval_executor::ExecutorContext,
    ) {
        // Pre
        state.current_played_ticks += 1;

        {
            let tempo = self.project_settings.read().unwrap().tempo;
            let tick = state.current_played_ticks;
            let sample_rate = state.sample_rate;
            let frames_per_cycle = state.buffer_size;
            let end_beat_time = tick_to_beat_time(tick, sample_rate, frames_per_cycle, tempo);

            let all_beats_in_frame = self.beat_sorter.write().unwrap().advance(end_beat_time);

            for (beat_time, beats) in all_beats_in_frame {
                let (tick, offset) =
                    beat_time_to_tick_and_offset(beat_time, sample_rate, frames_per_cycle, tempo);
                for (_, beat) in beats {
                    midi_adapter.write(offset as usize, beat);
                }
            }

            // BeatMakerSubscriptionModel::send_all(&state.beat_subscriber_map, event);
        }

        // Post
        state.last_n_frames += state.buffer_size;
    }
}

pub trait SSClient {
    fn start(&mut self) -> SSResult<()>;
    fn stop(&mut self) -> SSResult<()>;
}

pub trait SSClient2 {
    fn start(&mut self) -> SSResult<()>;
    fn stop(&mut self) -> SSResult<()>;
    fn pause(&mut self) -> SSResult<()>;
}

#[cfg(feature = "jack")]
pub fn create_ss_client(beatmaker_subscription: BeatMakerSubscription) -> Box<dyn SSClient + Send> {
    use self::jack::SSJackClient;
    Box::new(SSJackClient::new(beatmaker_subscription))
}

#[cfg(feature = "coreaudio")]
pub fn create_ss_client(beatmaker_subscription: BeatMakerSubscription) -> Box<dyn SSClient + Send> {
    use self::coreaudio::SSCoreAudioClient;
    Box::new(SSCoreAudioClient::new(beatmaker_subscription))
}

#[cfg(feature = "jack")]
pub fn create_ss_client(
    beatmaker_subscription: BeatMakerSubscription,
) -> Box<dyn SSClient2 + Send> {
    unimplemented!("TODO")
}

#[cfg(feature = "coreaudio")]
pub fn create_ss_client2(
    beat_sorter: Arc<RwLock<BeatSorter>>,
    project_settings: Arc<RwLock<ProjectSettings>>,
) -> Box<dyn SSClient2 + Send> {
    use self::coreaudio::SSCoreAudioClient2;
    Box::new(SSCoreAudioClient2::new(beat_sorter, project_settings))
}
