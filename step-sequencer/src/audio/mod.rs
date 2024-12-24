use crate::{beatmaker::BeatMaker, project::Project, SSResult};
#[cfg(target_os = "macos")]
mod coreaudio;
#[cfg(target_os = "linux")]
mod jack;

pub enum Command {
    ChangeTempo(u16)
}

pub trait SSClient {
    fn start(&mut self) -> SSResult<()>;
    fn stop(&mut self) -> SSResult<()>;
    fn send_command(&mut self, command: Command) -> SSResult<()>;
}

#[cfg(target_os = "linux")]
pub fn create_ss_client(beatmaker: BeatMaker, project: Project) -> SSResult<Box<dyn SSClient>> {
    use self::jack::SSJackClient;
    Ok(Box::new(SSJackClient::new(beatmaker, project)))
}

#[cfg(target_os = "macos")]
pub fn create_ss_client(beatmaker: BeatMaker, project: Project) -> SSResult<Box<dyn SSClient>> {
    use self::coreaudio::SSCoreAudioClient;
    Ok(Box::new(SSCoreAudioClient::new(beatmaker, project)))
}
