use crate::beatmaker::BeatMakerSubscription;
use crate::SSResult;
pub mod adapter;
#[cfg(feature = "coreaudio")]
mod coreaudio;
#[cfg(feature = "jack")]
mod jack;
#[cfg(feature = "cpal")]
mod cpal;

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

#[cfg(feature = "cpal")]
pub fn create_ss_client(beatmaker_subscription: BeatMakerSubscription) -> Box<dyn SSClient + Send> {
    use self::cpal::SSCpalClient;
    Box::new(SSCpalClient::new(beatmaker_subscription))
}
