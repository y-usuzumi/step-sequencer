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
use pattern::{play_example_pattern, BeatNoteMap};

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
    next_subscriber_id: u32,
}

impl Default for BeatMaker {
    fn default() -> Self {
        BeatMaker {
            subscribers: Arc::new(RwLock::new(HashMap::new())),
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

    pub fn start(&self, project: &Project) {
        let project_settings = project.project_settings();
        let tracks = project.tracks();
        let subscribers = self.subscribers.clone();
        thread::spawn(move || {
            info!("BeatMaker started");
            let beat_timer = BeatTimer::with_project_settings(project_settings);
            beat_timer.run_forever(|current_beats| {
                debug!("🥁 {}", current_beats);
                let subscribers = subscribers.read().unwrap();
                for track in tracks.read().unwrap().iter() {
                    let beat_idx = current_beats as usize % track.total_beats();
                    if let Some(beat) = track.get(beat_idx) {
                        send_beat(&subscribers, beat);
                    }
                }
            });
        });
    }

    #[deprecated(note = "Now for test only")]
    pub fn start_with_beat_note_map(
        &self,
        beat_note_map: BeatNoteMap,
    ) -> SSResult<BeatMakerAsyncHandle> {
        let subscribers = self.subscribers.clone();

        let thread_handle = thread::spawn(move || {
            play_example_pattern(&beat_note_map, subscribers);
        });
        Ok(BeatMakerAsyncHandle)
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
