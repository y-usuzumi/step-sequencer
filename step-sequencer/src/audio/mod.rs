use crate::beatmaker::BeatMakerSubscription;
use crate::SSResult;
#[cfg(feature = "coreaudio")]
mod coreaudio;
#[cfg(feature = "jack")]
mod jack;

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
