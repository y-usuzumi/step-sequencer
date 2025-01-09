use std::sync::RwLockReadGuard;

use log::debug;

use crate::{
    drum_track::{DrumTrack, DrumTrackBeat},
    midi::{note::Note, ChannelVoiceEvent, Key},
};

use crate::drum_track::Beat;
use crate::drum_track::DrumTrackBeat::*;

use super::BeatMakerSubscriberMap;

fn send_key<K>(subscribers: &RwLockReadGuard<BeatMakerSubscriberMap>, key: &K)
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

macro_rules! beat {
    ($channel:expr, $note:expr, $velocity:expr) => {
        Beat {
            channel: $channel,
            note: $note,
            velocity: $velocity,
        }
    };
}

pub trait ExampleDrumTracks {
    fn kick(&self) -> DrumTrack;
    fn snare(&self) -> DrumTrack;
    fn hihat(&self) -> DrumTrack;
    fn hihat_open(&self) -> DrumTrack;
    fn cymbal(&self) -> DrumTrack;
    fn all_tracks(&self) -> Vec<DrumTrack> {
        vec![
            self.kick(),
            self.snare(),
            self.hihat(),
            self.hihat_open(),
            self.cymbal(),
        ]
    }
}

pub struct ExampleDiscoDrumTracks {
    kick: Beat,
    snare: Beat,
    hihat: Beat,
    hihat_open: Beat,
    cymbal: Beat,
}

impl ExampleDrumTracks for ExampleDiscoDrumTracks {
    fn kick(&self) -> DrumTrack {
        DrumTrack::with_beats("Kick", self.kick, &[DefaultBeat, Unset].repeat(8))
    }

    fn snare(&self) -> DrumTrack {
        DrumTrack::with_beats(
            "Snare",
            self.snare,
            &[Unset, Unset, DefaultBeat, Unset].repeat(4),
        )
    }

    fn hihat(&self) -> DrumTrack {
        DrumTrack::with_beats("Hi-hat closed", self.hihat, &[DefaultBeat, Unset].repeat(8))
    }

    fn hihat_open(&self) -> DrumTrack {
        DrumTrack::with_beats(
            "Hi-hat open",
            self.hihat_open,
            &[Unset, DefaultBeat].repeat(8),
        )
    }

    fn cymbal(&self) -> DrumTrack {
        DrumTrack::with_beats(
            "Cymbal",
            self.cymbal,
            &[DefaultBeat]
                .into_iter()
                .chain([Unset].repeat(31))
                .collect::<Vec<DrumTrackBeat>>(),
        )
    }
}

pub static EXAMPLE_DRUMTRACKS_BITWIG: ExampleDiscoDrumTracks = ExampleDiscoDrumTracks {
    kick: beat!(9, Note::C(1), 72),
    snare: beat!(9, Note::Cs(1), 72),
    hihat: beat!(9, Note::D(1), 72),
    hihat_open: beat!(9, Note::Ds(1), 72),
    cymbal: beat!(9, Note::G(1), 72),
};

pub static EXAMPLE_DRUMTRACKS_GARAGEBAND: ExampleDiscoDrumTracks = ExampleDiscoDrumTracks {
    kick: beat!(9, Note::C(1), 72),
    snare: beat!(9, Note::Cs(1), 72),
    hihat: beat!(9, Note::Fs(1), 72),
    hihat_open: beat!(9, Note::As(1), 72),
    cymbal: beat!(9, Note::Cs(2), 72), // FIXME
};
