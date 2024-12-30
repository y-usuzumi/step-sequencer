use core::fmt;
use std::rc::Rc;

use crate::{beatmaker::BeatMaker, project::Project, SSResult};
#[cfg(target_os = "macos")]
mod coreaudio;
#[cfg(target_os = "linux")]
mod jack;

#[derive(Clone, Debug)]
pub enum Command {
    PlayOrPause,
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
pub fn create_ss_client(
    beatmaker: Rc<BeatMaker>,
    project: Rc<Project>,
) -> SSResult<Box<jack::SSJackClient>> {
    use self::jack::SSJackClient;
    Ok(Box::new(SSJackClient::new(beatmaker, project)))
}

#[cfg(target_os = "macos")]
pub fn create_ss_client(beatmaker: Rc<BeatMaker>) -> SSResult<Box<coreaudio::SSCoreAudioClient>> {
    use self::coreaudio::SSCoreAudioClient;
    Ok(Box::new(SSCoreAudioClient::new(beatmaker)))
}
