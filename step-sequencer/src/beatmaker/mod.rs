pub mod beat_timer;
pub mod pattern;

use std::{
    collections::HashMap,
    sync::{
        mpsc::{self},
        Arc, RwLock, RwLockReadGuard,
    },
    thread,
};

use beat_timer::BeatTimer;
use log::{debug, info};

use crate::{drum_track::Beat, midi::ChannelVoiceEvent, project::Project, SSResult};

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

type SubscriberMap = HashMap<u32, mpsc::Sender<ChannelVoiceEvent>>;

pub struct BeatMaker {
    subscribers: Arc<RwLock<SubscriberMap>>,
    beat_subscribers: Arc<RwLock<Vec<mpsc::Sender<u64>>>>,
    next_subscriber_id: u32,
}

impl Default for BeatMaker {
    fn default() -> Self {
        BeatMaker {
            subscribers: Arc::new(RwLock::new(HashMap::new())),
            beat_subscribers: Arc::new(RwLock::new(Vec::new())),
            next_subscriber_id: 0,
        }
    }
}

impl BeatMaker {
    pub fn subscribe(&mut self) -> BeatMakerSubscription {
        let mut subscriber_map = self.subscribers.write().unwrap();
        let (sender, receiver) = mpsc::channel();
        subscriber_map.insert(self.next_subscriber_id, sender);

        let subscription = BeatMakerSubscription {
            id: self.next_subscriber_id,
            receiver,
            subscribers: self.subscribers.clone(),
        };
        self.next_subscriber_id += 1;
        return subscription;
    }

    pub fn subscribe_beats(&self) -> mpsc::Receiver<u64> {
        let mut beat_subscribers = self.beat_subscribers.write().unwrap();
        let (sender, receiver) = mpsc::channel();
        beat_subscribers.push(sender);
        return receiver;
    }

    pub fn start(&self, project: &Project) {
        let project_settings = project.project_settings();
        let tracks = project.tracks();
        let subscribers = self.subscribers.clone();
        let beat_subscribers = self.beat_subscribers.clone();
        thread::spawn(move || {
            info!("BeatMaker started");
            let beat_timer = BeatTimer::with_project_settings(project_settings);
            beat_timer.run_forever(|current_beats| {
                debug!("🥁 {}", current_beats);
                {
                    let subscribers = subscribers.read().unwrap();
                    for track in tracks.read().unwrap().values() {
                        let beat_idx = current_beats as usize % track.total_beats();
                        if let Some(beat) = track.get_as_beat(beat_idx) {
                            send_beat(&subscribers, &beat);
                        }
                    }
                }
                {
                    let beat_subscribers = beat_subscribers.read().unwrap();
                    for beat_subscriber in beat_subscribers.iter() {
                        beat_subscriber.send(current_beats);
                    }
                }
            });
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
