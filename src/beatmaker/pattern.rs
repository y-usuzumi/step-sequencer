use std::{
    collections::HashMap,
    sync::{mpsc, Arc, Mutex, MutexGuard},
    thread,
    time::Duration,
};

use crate::midi::{ChannelVoiceEvent, Key};

fn send_key(subscribers: &MutexGuard<HashMap<u32, mpsc::Sender<ChannelVoiceEvent>>>, key: Key) {
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

#[derive(Debug, Clone)]
pub struct BeatNoteMap {
    pub kick: Key,
    pub hihat: Key,
    pub snare: Key,
    pub hihat_open: Key,
}

impl BeatNoteMap {
    fn kick(&self, subscribers: &MutexGuard<HashMap<u32, mpsc::Sender<ChannelVoiceEvent>>>) {
        send_key(subscribers, self.kick);
    }
    fn snare(&self, subscribers: &MutexGuard<HashMap<u32, mpsc::Sender<ChannelVoiceEvent>>>) {
        send_key(subscribers, self.snare);
    }
    fn hihat(&self, subscribers: &MutexGuard<HashMap<u32, mpsc::Sender<ChannelVoiceEvent>>>) {
        send_key(subscribers, self.hihat);
    }
    fn hihat_open(&self, subscribers: &MutexGuard<HashMap<u32, mpsc::Sender<ChannelVoiceEvent>>>) {
        send_key(subscribers, self.hihat_open);
    }
}

pub const BEAT_NOTE_MAP_BITWIG: BeatNoteMap = BeatNoteMap {
    kick: 36,
    snare: 37,
    hihat: 38,
    hihat_open: 39,
};
pub const BEAT_NOTE_MAP_GARAGEBAND: BeatNoteMap = BeatNoteMap {
    kick: 36,
    snare: 37,
    hihat: 42,
    hihat_open: 46,
};

pub fn play_example_pattern(
    beat_note_map: &BeatNoteMap,
    subscribers: Arc<Mutex<HashMap<u32, mpsc::Sender<ChannelVoiceEvent>>>>,
) {
    loop {
        let interval = 300;
        let subscribers = subscribers.lock().unwrap();
        // 1--
        beat_note_map.kick(&subscribers);
        beat_note_map.hihat(&subscribers);
        thread::sleep(Duration::from_millis(interval));
        beat_note_map.hihat(&subscribers);
        thread::sleep(Duration::from_millis(interval));
        beat_note_map.snare(&subscribers);
        beat_note_map.hihat(&subscribers);
        thread::sleep(Duration::from_millis(interval));
        beat_note_map.hihat(&subscribers);
        thread::sleep(Duration::from_millis(interval));
        // 2--
        beat_note_map.kick(&subscribers);
        beat_note_map.hihat(&subscribers);
        thread::sleep(Duration::from_millis(interval));
        beat_note_map.kick(&subscribers);
        beat_note_map.hihat(&subscribers);
        thread::sleep(Duration::from_millis(interval));
        beat_note_map.snare(&subscribers);
        beat_note_map.hihat(&subscribers);
        thread::sleep(Duration::from_millis(interval));
        beat_note_map.hihat(&subscribers);
        thread::sleep(Duration::from_millis(interval));
        // 3--
        beat_note_map.kick(&subscribers);
        beat_note_map.hihat(&subscribers);
        thread::sleep(Duration::from_millis(interval));
        beat_note_map.hihat(&subscribers);
        thread::sleep(Duration::from_millis(interval));
        beat_note_map.snare(&subscribers);
        beat_note_map.hihat(&subscribers);
        thread::sleep(Duration::from_millis(interval));
        beat_note_map.hihat(&subscribers);
        thread::sleep(Duration::from_millis(interval));
        // 4--
        beat_note_map.kick(&subscribers);
        beat_note_map.hihat(&subscribers);
        thread::sleep(Duration::from_millis(interval));
        beat_note_map.kick(&subscribers);
        beat_note_map.hihat(&subscribers);
        thread::sleep(Duration::from_millis(interval));
        beat_note_map.snare(&subscribers);
        beat_note_map.hihat(&subscribers);
        thread::sleep(Duration::from_millis(interval));
        beat_note_map.hihat_open(&subscribers);
        thread::sleep(Duration::from_millis(interval));
    }
}
