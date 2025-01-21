use std::{
    sync::{Arc, Condvar, Mutex, RwLock},
    thread,
    time::{Duration, Instant},
};

use log::warn;

type Tick = u64;

enum ExecutorState {
    Started,
    Paused,
    Stopped,
}

fn run_with_interval<F, State>(
    interval: Duration,
    mut callback: F,
    mut initial_state: State,
    start_immediately: bool,
) -> ExecutorHandle
where
    F: 'static + Send + FnMut(&mut State),
    State: 'static + Send,
{
    let current_tick = Arc::new(RwLock::new(0));
    let start_mutex = if start_immediately {
        Arc::new(Mutex::new(ExecutorState::Started))
    } else {
        Arc::new(Mutex::new(ExecutorState::Paused))
    };
    let state_condvar = Arc::new(Condvar::new());
    let cloned_current_tick = current_tick.clone();
    let cloned_start_mutex = start_mutex.clone();
    let cloned_state_condvar = state_condvar.clone();
    let _ = thread::spawn(move || {
        let start_mutex = start_mutex.clone();
        let current_tick = current_tick.clone();
        let mut next_tick_time = Instant::now() + interval;
        loop {
            if !*start_mutex.lock().unwrap() {
                state_condvar.notify_one();
                return;
            }
            let mut current_tick_locked = current_tick.write().unwrap();
            callback(&mut initial_state);
            *current_tick_locked += 1;
            next_tick_time += interval;
            {
                // FIXME: use plain mathematics instead of while loop
                let mut missed_ticks = 0;
                while Instant::now() > next_tick_time {
                    *current_tick_locked += 1;
                    next_tick_time += interval;
                    missed_ticks += 1;
                }
                if missed_ticks > 0 {
                    warn!("Skipping {} tick(s) due to slow processing", missed_ticks);
                }
            }

            thread::sleep(next_tick_time - Instant::now());
        }
    });
    ExecutorHandle {
        current_tick: cloned_current_tick,
        start_mutex: cloned_start_mutex,
        state_condvar: cloned_state_condvar,
    }
}

struct ExecutorHandle {
    /// This is only updated upon pause/stop
    current_tick: Arc<RwLock<Tick>>,
    start_mutex: Arc<Mutex<bool>>,
    state_condvar: Arc<Condvar>,
}

impl ExecutorHandle {
    pub fn pause(&self) {
        let mut guard = self.start_mutex.lock().unwrap();
        while *guard {
            *guard = false;
            guard = self.state_condvar.wait(guard).unwrap();
        }
    }

    pub fn resume(&self) {
        let mut guard = self.start_mutex.lock().unwrap();
        while *guard {
            *guard = true;
            guard = self.state_condvar.wait(guard).unwrap();
        }
    }

    pub fn stop(&self) {
        let mut guard = self.start_mutex.lock().unwrap();
        while *guard {
            *guard = false;
            guard = self.state_condvar.wait(guard).unwrap();
        }
        *self.current_tick.write().unwrap() = 0;
    }
}

impl Drop for ExecutorHandle {
    fn drop(&mut self) {
        self.
    }
}