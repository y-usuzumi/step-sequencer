use std::iter::repeat;

use log::debug;

use crate::{
    drum_track::{DrumTrack, DrumTrackBeat},
    midi::note::Note,
};

use crate::drum_track::Beat;
use crate::drum_track::DrumTrackBeat::*;

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
    fn blabla(&self) -> DrumTrack;

    fn all_tracks(&self) -> Vec<DrumTrack> {
        vec![
            self.kick(),
            self.snare(),
            self.hihat(),
            self.hihat_open(),
            self.cymbal(),
            self.blabla(),
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

fn repeat_slice<T: Clone>(slice: &[T], count: usize) -> Vec<T> {
    repeat(slice)
        .take(count)
        .flatten()
        .map(|elem| elem.clone())
        .collect()
}

impl ExampleDrumTracks for ExampleDiscoDrumTracks {
    fn kick(&self) -> DrumTrack {
        DrumTrack::with_beats("Kick", self.kick, &repeat_slice(&[DefaultBeat, Unset], 8))
    }

    fn snare(&self) -> DrumTrack {
        DrumTrack::with_beats(
            "Snare",
            self.snare,
            &repeat_slice(&[Unset, Unset, DefaultBeat, Unset], 4),
        )
    }

    fn hihat(&self) -> DrumTrack {
        DrumTrack::with_beats(
            "Hi-hat closed",
            self.hihat,
            &repeat_slice(&[DefaultBeat, Unset], 8),
        )
    }

    fn hihat_open(&self) -> DrumTrack {
        DrumTrack::with_beats(
            "Hi-hat open",
            self.hihat_open,
            &repeat_slice(&[Unset, DefaultBeat], 8),
        )
    }

    fn cymbal(&self) -> DrumTrack {
        DrumTrack::with_beats(
            "Cymbal",
            self.cymbal,
            &[DefaultBeat]
                .into_iter()
                .chain(repeat_slice(&[Unset], 31))
                .collect::<Vec<DrumTrackBeat>>(),
        )
    }

    fn blabla(&self) -> DrumTrack {
        DrumTrack::with_beats(
            "Blabla",
            self.cymbal,
            &[
                OverrideBeat(vec![beat!(1, Note::C(0), 127)]),
                OverrideBeat(vec![beat!(1, Note::Cs(0), 63)]),
                OverrideBeat(vec![beat!(1, Note::D(0), 63), beat!(1, Note::E(0), 63)]),
            ]
            .into_iter()
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
