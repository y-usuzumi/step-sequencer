use std::sync::{Arc, RwLock};

use log::info;
use util::{beat_time_to_tick_and_offset, tick_to_beat_time};

use crate::beatmaker::beat_sorter::BeatSorter;
use crate::beatmaker::{BeatMakerSubscriberMap, BeatMakerSubscription, BeatMakerSubscriptionModel};
use crate::project::ProjectSettings;
use crate::SSResult;
mod async_processor;
#[cfg(feature = "coreaudio")]
mod coreaudio;
#[cfg(feature = "jack")]
mod jack;
pub mod midi_adapter;
mod util;

pub trait SSClient {
    fn start(&mut self) -> SSResult<()>;
    fn stop(&mut self) -> SSResult<()>;
}

pub trait SSClient2 {
    fn start(&mut self) -> SSResult<()>;
    fn stop(&mut self) -> SSResult<()>;
    fn pause(&mut self) -> SSResult<()>;
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

#[cfg(feature = "coreaudio")]
pub fn create_ss_client2(
    beat_sorter: Arc<RwLock<BeatSorter>>,
    project_settings: Arc<RwLock<ProjectSettings>>,
) -> Box<dyn SSClient2 + Send> {
    use self::coreaudio::SSCoreAudioClient2;
    let client = SSCoreAudioClient2::new(beat_sorter, project_settings);
    Box::new(client)
}

#[cfg(feature = "jack")]
pub fn create_ss_client2(
    beat_sorter: Arc<RwLock<BeatSorter>>,
    project_settings: Arc<RwLock<ProjectSettings>>,
) -> Box<dyn SSClient2 + Send> {
    use self::jack::SSJackClient2;
    let client = SSJackClient2::new(beat_sorter, project_settings);
    Box::new(client)
}
