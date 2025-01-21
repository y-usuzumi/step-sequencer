use std::sync::Arc;

use log::info;

use crate::beatmaker::beat_sorter::BeatSorter;
use crate::beatmaker::BeatMakerSubscription;
use crate::SSResult;
pub mod adapter;
#[cfg(feature = "coreaudio")]
mod coreaudio;
#[cfg(feature = "jack")]
mod jack;

struct AsyncProcessor {
    beat_sorter: Arc<BeatSorter>,
}

struct AsyncProcessorState {
    process_cycle: usize,
    sample_rate: u64,
    buffer_size: u64,
    last_n_frames: u64,
}

impl AsyncProcessor {
    fn new(beat_sorter: Arc<BeatSorter>) -> Self {
        Self { beat_sorter }
    }
    fn on_process_cycle(
        &mut self,
        midi_adapter: &(dyn adapter::MIDIAdapter),
        state: &mut AsyncProcessorState,
        context: &crate::util::interval_executor::ExecutorContext,
    ) {
        state.process_cycle += 1;
        state.last_n_frames += state.buffer_size;
    }
}

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

#[cfg(feature = "jack")]
pub fn create_ss_client(
    beatmaker_subscription: BeatMakerSubscription,
) -> Box<dyn SSClient2 + Send> {
    unimplemented!("TODO")
}

#[cfg(feature = "coreaudio")]
pub fn create_ss_client2(beat_sorter: Arc<BeatSorter>) -> Box<dyn SSClient2 + Send> {
    use self::coreaudio::SSCoreAudioClient2;
    Box::new(SSCoreAudioClient2::new(beat_sorter))
}
