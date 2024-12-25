use std::{
    sync::{Arc, RwLock, RwLockReadGuard},
    thread,
    time::Duration,
};

use log::debug;

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
    debug!("BeatMaker: Sending events");
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

pub trait ExampleDrumTracks {
    fn kick(&self) -> DrumTrack;
    fn snare(&self) -> DrumTrack;
    fn hihat(&self) -> DrumTrack;
    fn hihat_open(&self) -> DrumTrack;
    fn all_tracks(&self) -> Vec<DrumTrack> {
        vec![self.kick(), self.snare(), self.hihat(), self.hihat_open()]
    }
}

pub struct ExampleDiscoDrumTracks {
    kick: Option<Beat>,
    snare: Option<Beat>,
    hihat: Option<Beat>,
    hihat_open: Option<Beat>,
}

impl ExampleDrumTracks for ExampleDiscoDrumTracks {
    fn kick(&self) -> DrumTrack {
        DrumTrack::with_beats(
            "Drum",
            &[
                self.kick, None, None, None, self.kick, self.kick, None, None,
            ],
        )
    }

    fn snare(&self) -> DrumTrack {
        DrumTrack::with_beats("Snare", &[None, None, self.snare, None])
    }

    fn hihat(&self) -> DrumTrack {
        DrumTrack::with_beats(
            "Hi-hat open",
            &[
                self.hihat, self.hihat, self.hihat, self.hihat, self.hihat, self.hihat, self.hihat,
                None,
            ],
        )
    }

    fn hihat_open(&self) -> DrumTrack {
        DrumTrack::with_beats(
            "Hi-hat closed",
            &[None, None, None, None, None, None, None, self.hihat_open],
        )
    }
}

pub static EXAMPLE_DRUMTRACKS_BITWIG: ExampleDiscoDrumTracks = ExampleDiscoDrumTracks {
    kick: beat!(9, Note::C(1), 72),
    snare: beat!(9, Note::Cs(1), 72),
    hihat: beat!(9, Note::D(1), 72),
    hihat_open: beat!(9, Note::Ds(1), 72),
};

pub static EXAMPLE_DRUMTRACKS_GARAGEBAND: ExampleDiscoDrumTracks = ExampleDiscoDrumTracks {
    kick: beat!(9, Note::C(1), 72),
    snare: beat!(9, Note::Cs(1), 72),
    hihat: beat!(9, Note::Fs(1), 72),
    hihat_open: beat!(9, Note::As(1), 72),
};
