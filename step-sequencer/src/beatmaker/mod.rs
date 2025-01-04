pub mod beat_timer;
pub mod pattern;

use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{
        mpsc::{self},
        Arc, RwLock, RwLockReadGuard,
    },
    thread,
};

use beat_timer::BeatTimerBuilder;
use log::{debug, info};

use crate::{
    drum_track::Beat, midi::ChannelVoiceEvent, project::Project, timeline::TimelineSubscription,
    SSResult,
};

fn send_beat(subscribers: &RwLockReadGuard<SubscriberMap>, beat: &Beat) {
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

type SubscriberMap = HashMap<u32, mpsc::Sender<ChannelVoiceEvent>>;
type SignalSubscriberMap = Vec<mpsc::Sender<BeatSignal>>;

pub struct BeatMaker {
    subscribers: Arc<RwLock<SubscriberMap>>,
    signal_subscribers: Arc<RwLock<SignalSubscriberMap>>,
    next_subscriber_id: RefCell<u32>,
}

impl BeatMaker {
    pub fn new() -> Self {
        BeatMaker {
            subscribers: Default::default(),
            signal_subscribers: Default::default(),
            next_subscriber_id: RefCell::new(0),
        }
    }
    pub fn subscribe(&self) -> BeatMakerSubscription {
        let mut subscriber_map = self.subscribers.write().unwrap();
        let (sender, receiver) = mpsc::channel();
        subscriber_map.insert(*self.next_subscriber_id.borrow(), sender);

        let subscription = BeatMakerSubscription {
            id: *self.next_subscriber_id.borrow(),
            receiver,
            subscribers: self.subscribers.clone(),
        };
        *self.next_subscriber_id.borrow_mut() += 1;
        return subscription;
    }

    pub fn subscribe_signals(&self) -> mpsc::Receiver<BeatSignal> {
        let mut signal_subscribers = self.signal_subscribers.write().unwrap();
        let (sender, receiver) = mpsc::channel();
        signal_subscribers.push(sender);
        return receiver;
    }

    pub fn start(&self, project: &Project, timeline_subscription: TimelineSubscription) {
        let project_settings = project.project_settings();
        let tracks = project.tracks();
        let subscribers = self.subscribers.clone();
        let signal_subscribers = self.signal_subscribers.clone();
        // let timeline_subscription = self.timeline.subscribe();
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

pub struct BeatMakerAsyncHandle;

pub struct BeatMakerSubscription {
    id: u32,
    receiver: mpsc::Receiver<ChannelVoiceEvent>,
    subscribers: Arc<RwLock<SubscriberMap>>,
}

impl BeatMakerSubscription {
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn recv(&self) -> SSResult<ChannelVoiceEvent> {
        match self.receiver.recv() {
            Ok(event) => Ok(event),
            Err(e) => Err(crate::error::SSError::Unknown(e.to_string())),
        }
    }

    pub fn try_recv(&self) -> SSResult<ChannelVoiceEvent> {
        match self.receiver.try_recv() {
            Ok(event) => Ok(event),
            Err(e) => Err(crate::error::SSError::Unknown(e.to_string())),
        }
    }

    pub fn iter(&self) -> mpsc::Iter<ChannelVoiceEvent> {
        self.receiver.iter()
    }
}

impl Drop for BeatMakerSubscription {
    fn drop(&mut self) {
        let mut subscriber_map = self.subscribers.write().unwrap();
        subscriber_map.remove(&self.id);
    }
}
