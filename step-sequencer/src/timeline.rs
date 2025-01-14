use std::{
    collections::BTreeMap,
    sync::{Arc, Condvar, Mutex, RwLock},
    thread,
    time::{Duration, Instant},
};

use crossbeam::channel::{bounded, Receiver, Sender};
use log::{info, warn};

use crate::id::{AutoIncrementId, AutoIncrementIdGen};

type Tick = u64;

pub enum TimelineState {
    Started,
    Stopped,
}

pub enum TimelineEvent {
    Tick(Tick),
    Pause,
    Stop,
}

type TimelineSubscriberMap = BTreeMap<AutoIncrementId, Sender<TimelineEvent>>;

/// Timeline is the heart of step-sequencer. It sends out ticks at precise interval.
/// BeatMaker's beat time is driven by the ticks, which ensures beat notes are sent
/// out at the proper time and in the correct order.
pub struct Timeline {
    interval: Duration,
    /// This is only updated upon pause/stop
    current_tick: Arc<RwLock<Tick>>,
    idgen: RwLock<AutoIncrementIdGen>,
    start_mutex: Arc<Mutex<bool>>,
    state_condvar: Arc<Condvar>,
    subscribers: Arc<RwLock<TimelineSubscriberMap>>,
}

impl Timeline {
    pub fn with_interval(interval: Duration) -> Self {
        Self {
            interval,
            ..Default::default()
        }
    }

    pub fn subscribe(&self) -> TimelineSubscription {
        let next_id = self.idgen.write().unwrap().next();
        let (sender, receiver) = bounded(5);
        self.subscribers.write().unwrap().insert(next_id, sender);
        return TimelineSubscription {
            id: next_id,
            interval: self.interval,
            receiver,
            subscribers: self.subscribers.clone(),
        };
    }

    pub fn state(&self) -> TimelineState {
        if *self.start_mutex.lock().unwrap() {
            TimelineState::Started
        } else {
            TimelineState::Stopped
        }
    }

    pub fn start(&self) {
        *self.start_mutex.lock().unwrap() = true;
        let start_mutex = self.start_mutex.clone();
        let state_condvar = self.state_condvar.clone();
        let subscribers = self.subscribers.clone();
        let last_current_tick = self.current_tick.clone();
        let interval = self.interval;
        let _ = thread::spawn(move || {
            let mut new_current_tick = *last_current_tick.read().unwrap();
            let mut next_tick_time = Instant::now() + interval;
            loop {
                if !*start_mutex.lock().unwrap() {
                    *last_current_tick.write().unwrap() = new_current_tick;
                    state_condvar.notify_one();
                    return;
                }
                for subscriber in subscribers.read().unwrap().values() {
                    subscriber
                        .send(TimelineEvent::Tick(new_current_tick))
                        .unwrap();
                }
                new_current_tick += 1;
                next_tick_time += interval;
                let now = Instant::now();
                if now > next_tick_time {
                    warn!("Skipping tick(s) due to slow processing");
                }
                while Instant::now() > next_tick_time {
                    new_current_tick += 1;
                    next_tick_time += interval;
                }

                thread::sleep(next_tick_time - Instant::now());
            }
        });
    }

    pub fn pause(&self) {
        let mut guard = self.start_mutex.lock().unwrap();
        while *guard {
            *guard = false;
            guard = self.state_condvar.wait(guard).unwrap();
        }
        for subscriber in self.subscribers.read().unwrap().values() {
            subscriber.send(TimelineEvent::Pause).unwrap();
        }
        info!("Timeline paused");
    }

    pub fn stop(&self) {
        let mut guard = self.start_mutex.lock().unwrap();
        while *guard {
            *guard = false;
            guard = self.state_condvar.wait(guard).unwrap();
        }
        for subscriber in self.subscribers.read().unwrap().values() {
            subscriber.send(TimelineEvent::Stop).unwrap();
        }
        info!("Timeline stopped");
        *self.current_tick.write().unwrap() = 0;
    }
}

impl Default for Timeline {
    fn default() -> Self {
        Self {
            interval: Duration::from_millis(10),
            current_tick: Default::default(),
            idgen: Default::default(),
            start_mutex: Default::default(),
            state_condvar: Default::default(),
            subscribers: Default::default(),
        }
    }
}

pub struct TimelineSubscription {
    pub id: AutoIncrementId,
    pub interval: Duration,
    pub receiver: Receiver<TimelineEvent>,
    subscribers: Arc<RwLock<TimelineSubscriberMap>>,
}

impl Drop for TimelineSubscription {
    fn drop(&mut self) {
        let mut subscriber_map = self.subscribers.write().unwrap();
        subscriber_map.remove(&self.id);
    }
}
