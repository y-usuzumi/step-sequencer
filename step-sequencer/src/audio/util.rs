//! Util module. Currently contains some functions for tick <-> BeatTime conversion

use crate::{
    beatmaker::beat_time::BeatTime,
    project::{Tempo, F},
};

pub fn tick_to_beat_time(
    tick: u64,
    sample_rate: u64,
    frames_per_cycle: u64,
    tempo: Tempo,
) -> BeatTime {
    BeatTime::new(F::from(tick) * F::from(tempo) * F::from(frames_per_cycle) / F::from(sample_rate))
}

pub fn beat_time_to_tick_and_offset(
    beat_time: BeatTime,
    sample_rate: u64,
    frames_per_cycle: u64,
    tempo: Tempo,
) -> (u64, u64) {
    let frames_f = beat_time.f() * F::from(sample_rate) / F::from(tempo);
    let tick_f = frames_f / F::from(frames_per_cycle);
    let tick = u64::try_from(tick_f.trunc()).unwrap();
    let offset = u64::try_from(frames_f).unwrap() % frames_per_cycle;
    (tick, offset)
}
