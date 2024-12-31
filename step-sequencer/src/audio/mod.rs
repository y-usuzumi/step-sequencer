use core::fmt;
use std::rc::Rc;

use crate::{beatmaker::BeatMaker, SSResult};
#[cfg(target_os = "macos")]
mod coreaudio;
#[cfg(target_os = "linux")]
mod jack;

#[derive(Clone, Debug)]
pub enum Command {
    PlayOrPause,
    Stop,
    ChangeTempo(u16),
    ToggleBeat(usize, usize),
    Resize(usize, usize),
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

#[cfg(target_os = "linux")]
pub fn create_ss_client(beatmaker: Rc<BeatMaker>) -> SSResult<Box<jack::SSJackClient>> {
    use self::jack::SSJackClient;
    Ok(Box::new(SSJackClient::new(beatmaker)))
}

#[cfg(target_os = "macos")]
pub fn create_ss_client(beatmaker: Rc<BeatMaker>) -> SSResult<Box<coreaudio::SSCoreAudioClient>> {
    use self::coreaudio::SSCoreAudioClient;
    Ok(Box::new(SSCoreAudioClient::new(beatmaker)))
}
