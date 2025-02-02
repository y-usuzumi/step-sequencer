use crate::audio::{AsyncProcessor, AsyncProcessorState, SSClient2};
use crate::beatmaker::beat_sorter::BeatSorter;
use crate::beatmaker::BeatMakerSubscriptionModel;
use crate::error::SSError;
use crate::project::ProjectSettings;
use crate::util::interval_executor::{run_with_interval, ExecutorHandle};
use crate::SSResult;

use coreaudio::audio_unit::render_callback::{self, data};
use coremidi::Client;
use crossbeam::channel::{bounded, Sender};
use log::{debug, info};
use std::ptr;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use super::midi_adapter::CoreAudioMIDIAdapter;

const SAMPLE_RATE: u64 = 44100;
const BUFFER_SIZE: u64 = 1024;

pub struct SSCoreAudioClient2 {
    beat_sorter: Arc<RwLock<BeatSorter>>,
    project_settings: Arc<RwLock<ProjectSettings>>,
    beat_subscription_model: BeatMakerSubscriptionModel,
    executor_handle: Option<ExecutorHandle>,
}

impl SSCoreAudioClient2 {
    pub fn new(
        beat_sorter: Arc<RwLock<BeatSorter>>,
        project_settings: Arc<RwLock<ProjectSettings>>,
    ) -> Self {
        Self {
            beat_sorter,
            project_settings,
            beat_subscription_model: BeatMakerSubscriptionModel::default(),
            executor_handle: None,
        }
    }
}

impl SSClient2 for SSCoreAudioClient2 {
    fn start(&mut self) -> SSResult<()> {
        if self.executor_handle.is_some() {
            return Err(SSError::Unknown("SSClient is started already".to_string()));
        }
        let client = Client::new("Yukio's Step Sequencer MIDI").unwrap();
        let source = client.virtual_source("source").unwrap();
        let nanos = 1_000_000_000 * BUFFER_SIZE / SAMPLE_RATE;
        let interval = Duration::from_nanos(nanos);
        let mut async_processor =
            AsyncProcessor::new(self.beat_sorter.clone(), self.project_settings.clone());
        let mut midi_adapter =
            CoreAudioMIDIAdapter::new(Arc::new(source), SAMPLE_RATE, BUFFER_SIZE as usize);
        let initial_state = AsyncProcessorState {
            sample_rate: SAMPLE_RATE,
            buffer_size: BUFFER_SIZE,
            last_n_frames: 0,
            current_played_ticks: 0,
        };
        let executor_handle = run_with_interval(
            interval,
            move |executor_context, state| {
                async_processor.on_process_cycle(&mut midi_adapter, state);
            },
            initial_state,
            true,
        );
        self.executor_handle = Some(executor_handle);
        Ok(())
    }

    fn pause(&mut self) -> SSResult<()> {
        self.executor_handle.as_ref().map(|h| h.pause());
        Ok(())
    }

    fn stop(&mut self) -> SSResult<()> {
        // No-op if interval_executor is not running.
        // Otherwise, take the underlying ExecutorHandle and drop it.
        self.executor_handle.take();
        Ok(())
    }
}
