use core::fmt;

use crate::{beatmaker::BeatMaker, project::Project, SSResult};
#[cfg(target_os = "macos")]
mod coreaudio;
#[cfg(target_os = "linux")]
mod jack;

#[derive(Clone, Debug)]
pub enum Command {
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
    fn start(&mut self) -> SSResult<()>;
    fn stop(&mut self) -> SSResult<()>;
    fn send_command(&self, command: Command) -> SSResult<()>;
}

#[cfg(target_os = "linux")]
pub fn create_ss_client<'a>(
    beatmaker: BeatMaker,
    project: &'a Project,
) -> SSResult<Box<dyn 'a + SSClient>> {
    use self::jack::SSJackClient;
    Ok(Box::new(SSJackClient::new(beatmaker, project)))
}

#[cfg(target_os = "macos")]
pub fn create_ss_client<'a>(
    beatmaker: BeatMaker,
    project: &'a Project,
) -> SSResult<Box<dyn 'a + SSClient>> {
    use self::coreaudio::SSCoreAudioClient;
    Ok(Box::new(SSCoreAudioClient::new(beatmaker, project)))
}
