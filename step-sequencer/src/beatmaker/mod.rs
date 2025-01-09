pub mod beat_sorter;
pub mod beat_timer;
pub mod pattern;

use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard},
    thread,
};

use beat_sorter::BeatSorter;
use beat_timer::BeatTimerBuilder;
use crossbeam::channel::{unbounded, Receiver, Sender};
use log::{debug, info};

use crate::{
    drum_track::Beat,
    id::{AutoIncrementId, AutoIncrementIdGen},
    midi::ChannelVoiceEvent,
    project::{BeatTime, Project, F},
    timeline::{TimelineEvent, TimelineSubscription},
};

fn send_beat(subscribers: &RwLockReadGuard<BeatMakerSubscriberMap>, beat: &Beat) {
    debug!("BeatMaker: Sending events");
    for sender in subscribers.values() {
        let _ = sender.send(ChannelVoiceEvent::NoteOn {
            channel: beat.channel,
            key: beat.note.into(),
            velocity: beat.velocity,
        });
        let _ = sender.send(ChannelVoiceEvent::NoteOff {
            channel: beat.channel,
            key: beat.note.into(),
            velocity: beat.velocity,
        });
    }
}

pub enum BeatSignal {
    Beat(BeatTime),
    Pause,
    Stop,
}

type BeatMakerSubscriberMap = HashMap<AutoIncrementId, Sender<ChannelVoiceEvent>>;
type SignalSubscriberMap = Vec<Sender<BeatSignal>>;

pub struct BeatMaker {
    subscribers: Arc<RwLock<BeatMakerSubscriberMap>>,
    signal_subscribers: Arc<RwLock<SignalSubscriberMap>>,
    idgen: RefCell<AutoIncrementIdGen>,
}

impl BeatMaker {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn subscribe(&self) -> BeatMakerSubscription {
        let next_id = self.idgen.borrow_mut().next();
        let (sender, receiver) = unbounded();
        let mut subscriber_map = self.subscribers.write().unwrap();
        subscriber_map.insert(next_id, sender);

        BeatMakerSubscription {
            id: next_id,
            receiver,
            subscribers: self.subscribers.clone(),
        }
    }

    pub fn subscribe_signals(&self) -> Receiver<BeatSignal> {
        let mut signal_subscribers = self.signal_subscribers.write().unwrap();
        let (sender, receiver) = unbounded();
        signal_subscribers.push(sender);
        return receiver;
    }

    pub fn start(&self, project: &Project, timeline_subscription: TimelineSubscription) {
        let project_settings = project.project_settings();
        let tracks = project.tracks();
        let subscribers = self.subscribers.clone();
        let signal_subscribers = self.signal_subscribers.clone();
        thread::spawn(move || {
            info!("BeatMaker started");
            let interval = timeline_subscription.interval;
            let mut beat_sorter = BeatSorter::new();
            let mut current_beat_time = (0usize, F::from(0));

            for tick in timeline_subscription.receiver.iter() {
                match tick {
                    TimelineEvent::Tick(_) => {
                        match beat_sorter.stored_next_beat() {
                            None => {
                                // First start or has been reset. Push all beats from global beat 0 to right before global beat 1
                                beat_sorter.push(0, &*tracks.read().unwrap());
                                beat_sorter.set_stored_next_beat(0);
                            }
                            Some(t) => {
                                // While we are processing the current beat i, get prepared for i+1 up till before i+2
                                if current_beat_time.0 >= t {
                                    beat_sorter
                                        .push(current_beat_time.0 + 1, &*tracks.read().unwrap());
                                    beat_sorter.set_stored_next_beat(current_beat_time.0 + 1);
                                }
                            }
                        }
                        let current_bpm = project_settings.read().unwrap().tempo;
                        current_beat_time.1 += F::from(interval.as_millis()) * current_bpm / 60_000;
                        if current_beat_time.1 > F::from(1) {
                            current_beat_time.0 +=
                                usize::try_from(current_beat_time.1.trunc()).unwrap();
                            current_beat_time.1 -= current_beat_time.1.trunc();
                        }
                        while beat_sorter
                            .next_beat_time()
                            .unwrap_or((usize::MAX, F::infinity()))
                            <= current_beat_time
                        {
                            let subscribers = subscribers.read().unwrap();
                            let signal_subscribers = signal_subscribers.read().unwrap();
                            if let Some((_, beats)) = beat_sorter.pop() {
                                beats.iter().flatten().for_each(|beat| {
                                    send_beat(&subscribers, beat);
                                });
                                for signal_subscriber in signal_subscribers.iter() {
                                    info!("🥁: ({}, {})", current_beat_time.0, current_beat_time.1);
                                    signal_subscriber.send(BeatSignal::Beat(current_beat_time));
                                }
                            } else {
                                panic!("Impossible! BeatSorter.pop() returns None while its next_beat_time() is Some()");
                            }
                        }
                        *project_settings
                            .read()
                            .unwrap()
                            .current_beat
                            .write()
                            .unwrap() = current_beat_time;
                    }
                    TimelineEvent::Pause => {
                        let signal_subscribers = signal_subscribers.read().unwrap();
                        for signal_subscriber in signal_subscribers.iter() {
                            signal_subscriber.send(BeatSignal::Pause);
                        }
                    }
                    TimelineEvent::Stop => {
                        *project_settings
                            .read()
                            .unwrap()
                            .current_beat
                            .write()
                            .unwrap() = (0, F::from(0));
                        current_beat_time = (0, F::from(0));
                        beat_sorter.reset();
                        let signal_subscribers = signal_subscribers.read().unwrap();
                        for signal_subscriber in signal_subscribers.iter() {
                            signal_subscriber.send(BeatSignal::Stop);
                        }
                    }
                }
            }
        });
    }
}

impl Default for BeatMaker {
    fn default() -> Self {
        Self {
            subscribers: Default::default(),
            signal_subscribers: Default::default(),
            idgen: Default::default(),
        }
    }
}

pub struct BeatMakerAsyncHandle;

pub struct BeatMakerSubscription {
    pub id: AutoIncrementId,
    pub receiver: Receiver<ChannelVoiceEvent>,
    pub subscribers: Arc<RwLock<BeatMakerSubscriberMap>>,
}

impl Drop for BeatMakerSubscription {
    fn drop(&mut self) {
        let mut subscriber_map = self.subscribers.write().unwrap();
        subscriber_map.remove(&self.id);
    }
}
