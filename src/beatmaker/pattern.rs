use std::{
    collections::HashMap,
    sync::{mpsc, Arc, Mutex, MutexGuard},
    thread,
    time::Duration,
};

use crate::midi::{note::Note, ChannelVoiceEvent, Key};

fn send_key<K>(subscribers: &MutexGuard<HashMap<u32, mpsc::Sender<ChannelVoiceEvent>>>, key: &K)
where
    K: Clone + Into<Key>,
{
    println!("BeatMaker: Sending events");
    for sender in subscribers.values() {
        let _ = sender.send(ChannelVoiceEvent::NoteOn {
            channel: 9, // is 10 to human
            key: key.clone().into(),
            velocity: 80,
        });
        let _ = sender.send(ChannelVoiceEvent::NoteOff {
            channel: 9, // is 10 to human
            key: key.clone().into(),
            velocity: 80,
        });
    }
}

#[derive(Debug, Clone)]
pub struct BeatNoteMap {
    pub kick: Note,
    pub hihat: Note,
    pub snare: Note,
    pub hihat_open: Note,
}

impl BeatNoteMap {
    fn kick(&self, subscribers: &MutexGuard<HashMap<u32, mpsc::Sender<ChannelVoiceEvent>>>) {
        send_key(subscribers, &self.kick);
    }
    fn snare(&self, subscribers: &MutexGuard<HashMap<u32, mpsc::Sender<ChannelVoiceEvent>>>) {
        send_key(subscribers, &self.snare);
    }
    fn hihat(&self, subscribers: &MutexGuard<HashMap<u32, mpsc::Sender<ChannelVoiceEvent>>>) {
        send_key(subscribers, &self.hihat);
    }
    fn hihat_open(&self, subscribers: &MutexGuard<HashMap<u32, mpsc::Sender<ChannelVoiceEvent>>>) {
        send_key(subscribers, &self.hihat_open);
    }
}

pub const BEAT_NOTE_MAP_BITWIG: BeatNoteMap = BeatNoteMap {
    kick: Note::C(1),
    snare: Note::Cs(1),
    hihat: Note::D(1),
    hihat_open: Note::Ds(1),
};
pub const BEAT_NOTE_MAP_GARAGEBAND: BeatNoteMap = BeatNoteMap {
    kick: Note::C(1),
    snare: Note::Cs(1),
    hihat: Note::Fs(1),
    hihat_open: Note::As(1),
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
