use std::sync::{Arc, RwLock};

use super::util::{beat_time_to_tick_and_offset, tick_to_beat_time};

use crate::beatmaker::beat_sorter::BeatSorter;
use crate::beatmaker::BeatMakerSubscriberMap;
use crate::project::ProjectSettings;

use super::midi_adapter;

pub struct AsyncProcessor {
    beat_sorter: Arc<RwLock<BeatSorter>>,
    project_settings: Arc<RwLock<ProjectSettings>>,
}

pub struct AsyncProcessorState {
    pub sample_rate: u64,
    pub buffer_size: u64,
    pub last_n_frames: u64,
    pub current_played_ticks: u64,
}

impl AsyncProcessor {
    pub fn new(
        beat_sorter: Arc<RwLock<BeatSorter>>,
        project_settings: Arc<RwLock<ProjectSettings>>,
    ) -> Self {
        Self {
            beat_sorter,
            project_settings,
        }
    }
    pub fn on_process_cycle(
        &mut self,
        midi_adapter: &mut (dyn midi_adapter::MIDIAdapter),
        state: &mut AsyncProcessorState,
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
        }

        // Post
        state.last_n_frames += state.buffer_size;
    }
}
