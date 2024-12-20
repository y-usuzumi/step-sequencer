use crate::SSResult;
#[cfg(target_os = "macos")]
mod coreaudio;
#[cfg(target_os = "linux")]
mod jack;

pub trait SSClient {
    fn start(&self) -> SSResult<()>;
}

#[cfg(target_os = "linux")]
pub fn get_ss_client() -> SSResult<Box<dyn SSClient>> {
    use self::jack::SSJackClient;
    Ok(Box::new(SSJackClient::new()))
}

#[cfg(target_os = "macos")]
pub fn get_ss_client() -> SSResult<Box<dyn SSClient>> {
    use self::coreaudio::SSCoreAudioClient;
    Ok(Box::new(SSCoreAudioClient::new()))
}
