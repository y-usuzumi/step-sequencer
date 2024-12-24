use crate::midi::{note::Note, Channel, Velocity};

pub struct DrumTrack {
    beats: Vec<Option<Beat>>,
}

#[derive(Debug, Clone)]
pub struct Beat {
    pub channel: Channel,
    pub note: Note,
    pub velocity: Velocity,
}

impl DrumTrack {
    pub fn new() -> Self {
        Self { beats: Vec::new() }
    }

    pub fn with_initial_beats(beats: usize) -> Self {
        Self {
            beats: vec![None; beats],
        }
    }

    pub fn with_beats(beats: &[Option<Beat>]) -> Self {
        Self {
            beats: beats.to_vec(),
        }
    }

    pub fn assign_beat(&mut self, idx: usize, beat: Beat) {
        if self.beats.len() < idx {
            self.beats.resize(idx + 1, None);
        }

        self.beats[idx] = Some(beat);
    }

    pub fn remove_beat(&mut self, idx: usize) {
        if let Some(v) = self.beats.get_mut(idx) {
            *v = None;
        }
    }

    pub fn set_total_beats(&mut self, idx: usize) {
        self.beats.resize(idx, None);
    }

    pub fn total_beats(&self) -> usize {
        self.beats.len()
    }

    pub fn get(&self, idx: usize) -> Option<&Beat> {
        self.beats.get(idx).unwrap_or(&None).as_ref()
    }
}
