use std::{
    collections::HashMap,
    sync::{
        mpsc::{self},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use crate::{midi::ChannelVoiceEvent, SSResult};

pub struct BeatMaker {
    bpm: u32,
    subscribers: Arc<Mutex<HashMap<u32, mpsc::Sender<ChannelVoiceEvent>>>>,
    id_counter: u32,
}

impl Default for BeatMaker {
    fn default() -> Self {
        BeatMaker {
            bpm: 110,
            subscribers: Arc::new(Mutex::new(HashMap::new())),
            id_counter: 0,
        }
    }
}

impl BeatMaker {
    pub fn subscribe(&mut self) -> BeatMakerSubscription {
        let mut subscriber_map = self.subscribers.lock().unwrap();
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

    pub fn start(&self) -> SSResult<BeatMakerAsyncHandle> {
        let subscribers = self.subscribers.clone();
        let thread_handle = thread::spawn(move || {
            for _ in 0..10 {
                for sender in subscribers.lock().unwrap().values() {
                    let _ = sender.send(ChannelVoiceEvent::NoteOff {
                        channel: 1,
                        key: 10,
                        velocity: 42,
                    });
                    thread::sleep(Duration::from_secs(3));
                }
            }
        });
        Ok(BeatMakerAsyncHandle)
    }
}

pub struct BeatMakerAsyncHandle;

pub struct BeatMakerSubscription {
    id: u32,
    receiver: mpsc::Receiver<ChannelVoiceEvent>,
    subscribers: Arc<Mutex<HashMap<u32, mpsc::Sender<ChannelVoiceEvent>>>>,
}

impl BeatMakerSubscription {
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
        let mut subscriber_map = self.subscribers.lock().unwrap();
        subscriber_map.remove(&self.id);
    }
}
