use crate::{
    midi::{note::Note, Channel, Velocity},
    project::{Tempo, F},
};

#[derive(Clone, Debug)]
pub enum Command {
    PlayOrPause,
    Stop,
    Quit,
    ChangeTempo(Tempo),
    AddTrack,
    RenameTrack(usize, String),
    ToggleBeat(usize, usize),
    Resize(usize, usize),
    TempoScale(usize, F),
    SetChannel(usize, Channel),
    SetVelocity(usize, Velocity),
    SetNote(usize, Note),
    Debug,
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
