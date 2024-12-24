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
use pattern::{play_example_pattern, BeatNoteMap};

use crate::{drum_track::Beat, midi::ChannelVoiceEvent, project::Project, SSResult};

fn send_beat(subscribers: &RwLockReadGuard<SubscriberMap>, beat: &Beat) {
    println!("BeatMaker: Sending events");
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
    bpm: u32,
    subscribers: Arc<RwLock<SubscriberMap>>,
    id_counter: u32,
}

impl Default for BeatMaker {
    fn default() -> Self {
        BeatMaker {
            bpm: 110,
            subscribers: Arc::new(RwLock::new(HashMap::new())),
            id_counter: 0,
        }
    }
}

impl BeatMaker {
    pub fn subscribe(&mut self) -> BeatMakerSubscription {
        let mut subscriber_map = self.subscribers.write().unwrap();
        let (sender, receiver) = mpsc::channel();
        subscriber_map.insert(self.id_counter, sender);

        let subscription = BeatMakerSubscription {
            id: self.id_counter,
            receiver,
            subscribers: self.subscribers.clone(),
        };
        self.id_counter += 1;
        return subscription;
    }

    pub fn start(&self, project: &Project) {
        let project_settings = project.project_settings();
        let tempo = project_settings.read().unwrap().tempo;
        let beat_timer = BeatTimer::with_bpm(tempo);
        let tracks = project.tracks();
        let subscribers = self.subscribers.clone();
        thread::spawn(move || {
            println!("BeatMaker started");
            beat_timer.run_forever(|current_beats| {
                println!("🥁 {}", current_beats);
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
