use std::{
    sync::{Arc, Condvar, Mutex, RwLock},
    thread,
    time::{Duration, Instant},
};

use crossbeam::channel::bounded;
use log::{info, warn};

use crate::{
    consts,
    models::channel_subscription::{ChannelEventSubscription, ChannelEventSubscriptionModel},
};

type Tick = u64;

pub enum TimelineState {
    Started,
    Stopped,
}

#[derive(Copy, Clone, PartialEq)]
pub enum TimelineEvent {
    Tick(Tick),
    Pause,
    Stop,
}

pub type TimelineSubscriptionModel = ChannelEventSubscriptionModel<TimelineEvent>;
pub type TimelineSubscription = ChannelEventSubscription<TimelineEvent>;

/// Timeline is the heart of step-sequencer. It sends out ticks at precise interval.
/// BeatMaker's beat time is driven by the ticks, which ensures beat notes are sent
/// out at the proper time and in the correct order.
pub struct Timeline {
    interval: Duration,
    /// This is only updated upon pause/stop
    current_tick: Arc<RwLock<Tick>>,
    start_mutex: Arc<Mutex<bool>>,
    state_condvar: Arc<Condvar>,
    subscription_model: TimelineSubscriptionModel,
}

impl Timeline {
    pub fn new() -> Self {
        Self {
            interval: consts::TIMELINE_TICK_DURATION,
            ..Default::default()
        }
    }

    pub fn subscribe(&self) -> TimelineSubscription {
        self.subscription_model.subscribe()
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
        let last_current_tick = self.current_tick.clone();
        let interval = self.interval;
        let subscriber_map = self.subscription_model.subscriber_map().clone();
        let _ = thread::spawn(move || {
            let mut new_current_tick = *last_current_tick.read().unwrap();
            let mut next_tick_time = Instant::now() + interval;
            loop {
                if !*start_mutex.lock().unwrap() {
                    *last_current_tick.write().unwrap() = new_current_tick;
                    state_condvar.notify_one();
                    return;
                }
                ChannelEventSubscriptionModel::send_all(
                    &subscriber_map,
                    TimelineEvent::Tick(new_current_tick),
                );
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
        let subscriber_map = self.subscription_model.subscriber_map();
        ChannelEventSubscriptionModel::send_all(subscriber_map, TimelineEvent::Pause);
        info!("Timeline paused");
    }

    pub fn stop(&self) {
        let mut guard = self.start_mutex.lock().unwrap();
        while *guard {
            *guard = false;
            guard = self.state_condvar.wait(guard).unwrap();
        }

        let subscriber_map = self.subscription_model.subscriber_map();
        ChannelEventSubscriptionModel::send_all(subscriber_map, TimelineEvent::Stop);

        info!("Timeline stopped");
        *self.current_tick.write().unwrap() = 0;
    }
}

impl Default for Timeline {
    fn default() -> Self {
        Self {
            interval: Duration::from_millis(10),
            current_tick: Default::default(),
            start_mutex: Default::default(),
            state_condvar: Default::default(),
            subscription_model: TimelineSubscriptionModel::new(|| bounded(5)),
        }
    }
}
