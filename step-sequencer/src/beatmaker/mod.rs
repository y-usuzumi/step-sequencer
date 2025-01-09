pub mod beat_timer;
pub mod pattern;

use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard},
    thread,
};

use beat_timer::BeatTimerBuilder;
use crossbeam::channel::{unbounded, Receiver, Sender};
use log::{debug, info};

use crate::{
    drum_track::Beat,
    id::{AutoIncrementId, AutoIncrementIdGen},
    midi::ChannelVoiceEvent,
    project::Project,
    timeline::TimelineSubscription,
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
    Beat(u64),
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
            let beat_timer = BeatTimerBuilder::default()
                .timeline_subscription(timeline_subscription)
                .project_settings(project_settings)
                .build()
                .unwrap();
            beat_timer.run_forever(
                |current_beats| {
                    info!("🥁 {}", current_beats);
                    {
                        let subscribers = subscribers.read().unwrap();
                        for track in tracks.read().unwrap().values() {
                            let beat_idx = current_beats as usize % track.len();
                            if let Some(Some(beat)) = track.get_as_beat(beat_idx) {
                                send_beat(&subscribers, &beat);
                            }
                        }
                    }
                    {
                        let signal_subscribers = signal_subscribers.read().unwrap();
                        for signal_subscriber in signal_subscribers.iter() {
                            signal_subscriber.send(BeatSignal::Beat(current_beats));
                        }
                    }
                },
                || {
                    let signal_subscribers = signal_subscribers.read().unwrap();
                    for signal_subscriber in signal_subscribers.iter() {
                        signal_subscriber.send(BeatSignal::Pause);
                    }
                },
                || {
                    let signal_subscribers = signal_subscribers.read().unwrap();
                    for signal_subscriber in signal_subscribers.iter() {
                        signal_subscriber.send(BeatSignal::Stop);
                    }
                },
            );
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
