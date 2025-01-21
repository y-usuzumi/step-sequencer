use crate::audio::{AsyncProcessor, AsyncProcessorState, SSClient2};
use crate::beatmaker::beat_sorter::BeatSorter;
use crate::error::SSError;
use crate::util::interval_executor::{run_with_interval, ExecutorHandle};
use crate::SSResult;

use coreaudio::audio_unit::render_callback::{self, data};
use coreaudio::audio_unit::{AudioUnit, IOType, SampleFormat};
use coremidi::Client;
use crossbeam::channel::{bounded, Sender};
use log::{debug, info};
use std::ptr;
use std::sync::Arc;
use std::time::Duration;

use super::CoreAudioMIDIAdapter;

const SAMPLE_RATE: u64 = 44100;
const BUFFER_SIZE: u64 = 1024;

pub struct SSCoreAudioClient2 {
    beat_sorter: Arc<BeatSorter>,
    executor_handle: Option<ExecutorHandle>,
}

impl SSCoreAudioClient2 {
    pub fn new(beat_sorter: Arc<BeatSorter>) -> Self {
        Self {
            beat_sorter,
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
        let mut async_processor = AsyncProcessor::new(self.beat_sorter.clone());
        let midi_adapter = CoreAudioMIDIAdapter {
            virtual_source: Arc::new(source),
            buffer_size: BUFFER_SIZE as usize,
            sample_rate: SAMPLE_RATE,
            nanosecs_on_play: 0,
            last_n_frames: 0,
        };
        let initial_state = AsyncProcessorState {
            process_cycle: 0,
            sample_rate: SAMPLE_RATE,
            buffer_size: BUFFER_SIZE,
            last_n_frames: 0,
        };
        let executor_handle = run_with_interval(
            interval,
            move |executor_context, state| {
                async_processor.on_process_cycle(&midi_adapter, state, executor_context);
            },
            initial_state,
            true,
        );
        self.executor_handle = Some(executor_handle);
        Ok(())
    }

    fn stop(&mut self) -> SSResult<()> {
        // No-op if interval_executor is not running.
        // Otherwise, take the underlying ExecutorHandle and drop it.
        self.executor_handle.take();
        Ok(())
    }

    fn pause(&mut self) -> SSResult<()> {
        self.executor_handle.as_ref().map(|h| h.pause());
        Ok(())
    }
}
