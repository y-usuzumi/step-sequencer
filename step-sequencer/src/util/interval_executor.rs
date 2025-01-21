use self::ExecutorState::*;
use std::{
    sync::{Arc, Condvar, Mutex, RwLock, RwLockReadGuard},
    thread,
    time::{Duration, Instant},
};

use log::warn;

type Tick = u64;

#[derive(Debug, Clone, Copy, PartialEq)]
enum ExecutorState {
    Started,
    Paused,
    Terminated,
}

pub(crate) struct ExecutorContext {
    current_tick: Tick,
}

pub(crate) fn run_with_interval<F, State>(
    interval: Duration,
    mut callback: F,
    mut initial_state: State,
    start_immediately: bool,
) -> ExecutorHandle
where
    F: 'static + Send + FnMut(&ExecutorContext, &mut State),
    State: 'static + Send,
{
    let current_context = Arc::new(RwLock::new(ExecutorContext { current_tick: 0 }));
    let start_mutex = if start_immediately {
        Arc::new(Mutex::new(ExecutorState::Started))
    } else {
        Arc::new(Mutex::new(ExecutorState::Paused))
    };
    let state_condvar = Arc::new(Condvar::new());
    let cloned_current_context = current_context.clone();
    let cloned_start_mutex = start_mutex.clone();
    let cloned_state_condvar = state_condvar.clone();
    println!("Spawning thread");
    let _ = thread::spawn(move || {
        let start_mutex = start_mutex.clone();
        let mut next_tick_time = Instant::now() + interval;
        loop {
            {
                let mutex_guard = start_mutex.lock().unwrap();
                match *mutex_guard {
                    Started => {}
                    Paused => {
                        continue;
                    }
                    Terminated => {
                        println!("Terminated");
                        state_condvar.notify_one();
                        return;
                    }
                }
            }
            let mut current_context_locked = current_context.write().unwrap();
            callback(&*current_context_locked, &mut initial_state);
            current_context_locked.current_tick += 1;
            next_tick_time += interval;
            {
                // FIXME: use plain mathematics instead of while loop
                let mut missed_ticks = 0;
                while Instant::now() > next_tick_time {
                    current_context_locked.current_tick += 1;
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
        current_context: cloned_current_context,
        start_mutex: cloned_start_mutex,
        state_condvar: cloned_state_condvar,
    }
}

pub struct ExecutorHandle {
    /// This is only updated upon pause/stop
    current_context: Arc<RwLock<ExecutorContext>>,
    start_mutex: Arc<Mutex<ExecutorState>>,
    state_condvar: Arc<Condvar>,
}

impl ExecutorHandle {
    pub fn state(&self) -> ExecutorState {
        return *self.start_mutex.lock().unwrap();
    }

    pub fn current_context(&self) -> RwLockReadGuard<ExecutorContext> {
        return self.current_context.read().unwrap();
    }

    pub fn pause(&self) {
        let mut guard = self.start_mutex.lock().unwrap();
        while *guard != Paused {
            *guard = Paused;
        }
    }

    pub fn resume(&self) {
        let mut guard = self.start_mutex.lock().unwrap();
        *guard = Started;
    }

    pub fn stop(&self) {
        let mut guard = self.start_mutex.lock().unwrap();
        *guard = Paused;
        self.current_context.write().unwrap().current_tick = 0;
    }
}

impl Drop for ExecutorHandle {
    fn drop(&mut self) {
        println!("Dropping");
        let mut guard = self.start_mutex.lock().unwrap();
        while *guard != Terminated {
            *guard = Terminated;
            self.state_condvar.notify_one();
            guard = self.state_condvar.wait(guard).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use crate::util::interval_executor::ExecutorState;

    use super::run_with_interval;

    #[test]
    fn test_manual_start() {
        let (send, recv) = crossbeam::channel::bounded(1);
        let handle = run_with_interval(
            Duration::from_millis(2),
            move |ctx, _state| {
                send.send(ctx.current_tick).unwrap();
            },
            (),
            false,
        );
        assert_eq!(handle.state(), ExecutorState::Paused);
        handle.resume();
        assert_eq!(handle.state(), ExecutorState::Started);
        assert_eq!(recv.recv().unwrap(), 0);

        // If I pause immediately, the executor will not have the chance to send the second tick because of the 2ms delay.
        // Therefore I'm waiting for a bit longer
        thread::sleep(Duration::from_millis(5));
        handle.pause();
        assert_eq!(handle.state(), ExecutorState::Paused);
        assert_eq!(recv.recv().unwrap(), 1); // Second tick
        assert_eq!(handle.state(), ExecutorState::Paused);
        // Thread is paused and no longer sending ticks
        // Wait for the next iteration where the thread is blocked at wait() for CondVar
        thread::sleep(Duration::from_millis(5));
        assert!(recv.try_recv().is_err());

        handle.resume();
        assert_eq!(handle.state(), ExecutorState::Started);
        assert_eq!(recv.recv().unwrap(), 2); // Third tick
        assert_eq!(recv.recv().unwrap(), 5); // Fourth tick, underrun
        assert_eq!(recv.recv().unwrap(), 6); // Receive a few more
        handle.stop();
        assert_eq!(handle.state(), ExecutorState::Paused);
        // Stop will reset current tick
        assert_eq!(handle.current_context().current_tick, 0);
        handle.resume();
        assert_eq!(handle.state(), ExecutorState::Started);
        assert_eq!(recv.recv().unwrap(), 0); // First tick after stop
        handle.pause();
        assert_eq!(handle.state(), ExecutorState::Paused);
        // Stop should NOT block after pause
        handle.stop();
        assert_eq!(handle.state(), ExecutorState::Paused);
    }
}
