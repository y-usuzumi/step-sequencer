use std::{
    sync::{Arc, RwLock},
    thread,
    time::{Duration, Instant},
};

pub struct BeatTimer {
    interval: Arc<RwLock<Duration>>,
}

fn bpm_to_duration(beats: u16) -> Duration {
    Duration::from_secs_f64(60. / (beats as f64))
}

impl BeatTimer {
    pub fn with_interval(interval: Duration) -> Self {
        Self {
            interval: Arc::new(RwLock::new(interval)),
        }
    }

    pub fn with_bpm(bpm: u16) -> Self {
        Self::with_interval(bpm_to_duration(bpm))
    }

    pub fn set_interval(&mut self, interval: Duration) {
        let mut duration = self.interval.write().unwrap();
        *duration = interval;
    }

    pub fn run_forever<T>(&self, on_beat: T)
    where
        T: Fn(u64),
    {
        let mut current_beat = 0;
        let mut next_time = Instant::now() + *self.interval.read().unwrap();
        loop {
            on_beat(current_beat);
            thread::sleep(next_time - Instant::now());
            next_time += *self.interval.read().unwrap();
            current_beat += 1;
        }
    }
}

impl Default for BeatTimer {
    fn default() -> Self {
        Self::with_bpm(110)
    }
}
