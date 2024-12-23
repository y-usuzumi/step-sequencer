use std::{
    collections::HashMap,
    sync::{
        mpsc::{self},
        Arc, Mutex, MutexGuard,
    },
    thread,
    time::Duration,
};

use crate::{
    midi::{ChannelVoiceEvent, Key},
    SSResult,
};

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
        fn send_key(
            subscribers: &MutexGuard<HashMap<u32, mpsc::Sender<ChannelVoiceEvent>>>,
            key: Key,
        ) {
            println!("BeatMaker: Sending events");
            for sender in subscribers.values() {
                let _ = sender.send(ChannelVoiceEvent::NoteOn {
                    channel: 9, // is 10 to human
                    key: key,
                    velocity: 80,
                });
                let _ = sender.send(ChannelVoiceEvent::NoteOff {
                    channel: 9, // is 10 to human
                    key: key,
                    velocity: 80,
                });
            }
        }
        fn kick(subscribers: &MutexGuard<HashMap<u32, mpsc::Sender<ChannelVoiceEvent>>>) {
            send_key(subscribers, 36);
        }
        fn snare(subscribers: &MutexGuard<HashMap<u32, mpsc::Sender<ChannelVoiceEvent>>>) {
            send_key(subscribers, 37);
        }
        fn hihat(subscribers: &MutexGuard<HashMap<u32, mpsc::Sender<ChannelVoiceEvent>>>) {
            send_key(subscribers, 38);
        }
        fn hihat_open(subscribers: &MutexGuard<HashMap<u32, mpsc::Sender<ChannelVoiceEvent>>>) {
            send_key(subscribers, 39);
        }
        let thread_handle = thread::spawn(move || loop {
            let interval = 300;
            let subscribers = subscribers.lock().unwrap();
            // 1--
            kick(&subscribers);
            hihat(&subscribers);
            thread::sleep(Duration::from_millis(interval));
            hihat(&subscribers);
            thread::sleep(Duration::from_millis(interval));
            snare(&subscribers);
            hihat(&subscribers);
            thread::sleep(Duration::from_millis(interval));
            hihat(&subscribers);
            thread::sleep(Duration::from_millis(interval));
            // 2--
            kick(&subscribers);
            hihat(&subscribers);
            thread::sleep(Duration::from_millis(interval));
            kick(&subscribers);
            hihat(&subscribers);
            thread::sleep(Duration::from_millis(interval));
            snare(&subscribers);
            hihat(&subscribers);
            thread::sleep(Duration::from_millis(interval));
            hihat(&subscribers);
            thread::sleep(Duration::from_millis(interval));
            // 3--
            kick(&subscribers);
            hihat(&subscribers);
            thread::sleep(Duration::from_millis(interval));
            hihat(&subscribers);
            thread::sleep(Duration::from_millis(interval));
            snare(&subscribers);
            hihat(&subscribers);
            thread::sleep(Duration::from_millis(interval));
            hihat(&subscribers);
            thread::sleep(Duration::from_millis(interval));
            // 4--
            kick(&subscribers);
            hihat(&subscribers);
            thread::sleep(Duration::from_millis(interval));
            kick(&subscribers);
            hihat(&subscribers);
            thread::sleep(Duration::from_millis(interval));
            snare(&subscribers);
            hihat(&subscribers);
            thread::sleep(Duration::from_millis(interval));
            hihat_open(&subscribers);
            thread::sleep(Duration::from_millis(interval));
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
        let mut subscriber_map = self.subscribers.lock().unwrap();
        subscriber_map.remove(&self.id);
    }
}
