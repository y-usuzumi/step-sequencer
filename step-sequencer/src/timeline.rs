use std::{
    cell::RefCell,
    collections::BTreeMap,
    sync::{mpsc, Arc, Mutex, RwLock},
    thread,
    time::{Duration, Instant},
};

use log::warn;

use crate::id::{AutoIncrementId, AutoIncrementIdGen};

type Tick = u64;

pub enum TimelineState {
    Started,
    Stopped,
}

pub enum TimelineEvent {
    Tick(Tick),
    Stop,
}

pub struct Timeline {
    interval: Duration,
    /// This is only updated upon pause/stop
    current_tick: Arc<RwLock<Tick>>,
    idgen: RefCell<AutoIncrementIdGen>,
    start_mutex: Arc<Mutex<bool>>,
    subscribers: Arc<RwLock<BTreeMap<AutoIncrementId, mpsc::SyncSender<TimelineEvent>>>>,
}

impl Timeline {
    pub fn with_interval(interval: Duration) -> Self {
        Self {
            interval,
            ..Default::default()
        }
    }

    pub fn subscribe(&self) -> TimelineSubscription {
        let next_id = self.idgen.borrow_mut().next();
        let (sender, receiver) = mpsc::sync_channel(5);
        self.subscribers.write().unwrap().insert(next_id, sender);
        return TimelineSubscription {
            id: next_id,
            interval: self.interval,
            receiver,
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
        let subscribers = self.subscribers.clone();
        let last_current_tick = self.current_tick.clone();
        let interval = self.interval;
        let _ = thread::spawn(move || {
            let mut new_current_tick = *last_current_tick.read().unwrap();
            let mut next_tick_time = Instant::now() + interval;
            loop {
                if !*start_mutex.lock().unwrap() {
                    for subscriber in subscribers.read().unwrap().values() {
                        subscriber.send(TimelineEvent::Stop).unwrap();
                    }

                    *last_current_tick.write().unwrap() = new_current_tick;
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
        *self.start_mutex.lock().unwrap() = false;
    }

    pub fn stop(&mut self) {
        let mut mutexguard = self.start_mutex.lock().unwrap();
        *mutexguard = false;
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
            subscribers: Default::default(),
        }
    }
}

pub struct TimelineSubscription {
    pub id: AutoIncrementId,
    pub interval: Duration,
    pub receiver: mpsc::Receiver<TimelineEvent>,
}
