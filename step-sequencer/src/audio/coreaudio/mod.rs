mod midi_adapter;
mod step_sequencer_client;
mod step_sequencer_client2;
mod util;

use std::sync::Arc;

use coremidi::{PacketBuffer, VirtualSource};
use util::nanosecs_to_mach_ticks;

use crate::SSResult;

pub use step_sequencer_client::SSCoreAudioClient;
pub use step_sequencer_client2::SSCoreAudioClient2;
