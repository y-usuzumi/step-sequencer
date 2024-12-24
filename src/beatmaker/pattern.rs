use std::{
    sync::{Arc, RwLock, RwLockReadGuard},
    thread,
    time::Duration,
};

use crate::{
    drum_track::DrumTrack,
    midi::{note::Note, ChannelVoiceEvent, Key},
};

use crate::drum_track::Beat;

use super::SubscriberMap;

fn send_key<K>(subscribers: &RwLockReadGuard<SubscriberMap>, key: &K)
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
    fn kick(&self, subscribers: &RwLockReadGuard<SubscriberMap>) {
        send_key(subscribers, &self.kick);
    }
    fn snare(&self, subscribers: &RwLockReadGuard<SubscriberMap>) {
        send_key(subscribers, &self.snare);
    }
    fn hihat(&self, subscribers: &RwLockReadGuard<SubscriberMap>) {
        send_key(subscribers, &self.hihat);
    }
    fn hihat_open(&self, subscribers: &RwLockReadGuard<SubscriberMap>) {
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

pub fn play_example_pattern(beat_note_map: &BeatNoteMap, subscribers: Arc<RwLock<SubscriberMap>>) {
    loop {
        let interval = 300;
        let subscribers = subscribers.read().unwrap();
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

macro_rules! beat {
    ($channel:expr, $note:expr, $velocity:expr) => {
        Some(Beat {
            channel: $channel,
            note: $note,
            velocity: $velocity,
        })
    };
}

pub fn create_example_track_kick_snare() -> DrumTrack {
    DrumTrack::with_beats(&[
        beat!(9, Note::C(1), 80),
        None,
        beat!(9, Note::Cs(1), 80),
        None,
        beat!(9, Note::C(1), 80),
        beat!(9, Note::C(1), 80),
        beat!(9, Note::Cs(1), 80),
        None,
    ])
}

pub fn create_example_track_hihat() -> DrumTrack {
    DrumTrack::with_beats(&[
        beat!(9, Note::D(1), 80),
        beat!(9, Note::D(1), 80),
        beat!(9, Note::D(1), 80),
        beat!(9, Note::D(1), 80),
        beat!(9, Note::D(1), 80),
        beat!(9, Note::D(1), 80),
        beat!(9, Note::D(1), 80),
        beat!(9, Note::Ds(1), 80),
    ])
}
