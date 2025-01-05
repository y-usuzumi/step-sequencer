use core::fmt;
use std::rc::Rc;

use crate::{
    beatmaker::BeatMaker,
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
    ChangeTempo(u16),
    ToggleBeat(usize, usize),
    Resize(usize, usize),
    SetChannel(usize, Channel),
    SetVelocity(usize, Velocity),
    SetNote(usize, Note),
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub trait SSClient {
    fn start(&self) -> SSResult<()>;
    fn stop(&self) -> SSResult<()>;
}

#[cfg(feature = "jack")]
pub fn create_ss_client(beatmaker: Rc<BeatMaker>) -> SSResult<Box<jack::SSJackClient>> {
    use self::jack::SSJackClient;
    Ok(Box::new(SSJackClient::new(beatmaker)))
}

#[cfg(feature = "coreaudio")]
pub fn create_ss_client(beatmaker: Rc<BeatMaker>) -> SSResult<Box<coreaudio::SSCoreAudioClient>> {
    use self::coreaudio::SSCoreAudioClient;
    Ok(Box::new(SSCoreAudioClient::new(beatmaker)))
}
