use core::fmt;

use crate::beatmaker::BeatMakerSubscription;
use crate::project::{Tempo, F};
use crate::{
    midi::{note::Note, Channel, Velocity},
    SSResult,
};
#[cfg(feature = "coreaudio")]
mod coreaudio;
#[cfg(feature = "jack")]
mod jack;

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

impl fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub trait SSClient {
    fn start(&mut self) -> SSResult<()>;
    fn stop(&mut self) -> SSResult<()>;
}

#[cfg(feature = "jack")]
pub fn create_ss_client(beatmaker_subscription: BeatMakerSubscription) -> Box<dyn SSClient + Send> {
    use self::jack::SSJackClient;
    Box::new(SSJackClient::new(beatmaker_subscription))
}

#[cfg(feature = "coreaudio")]
pub fn create_ss_client(beatmaker_subscription: BeatMakerSubscription) -> Box<dyn SSClient + Send> {
    use self::coreaudio::SSCoreAudioClient;
    Box::new(SSCoreAudioClient::new(beatmaker_subscription))
}
