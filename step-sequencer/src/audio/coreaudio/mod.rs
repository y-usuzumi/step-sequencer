mod step_sequencer_client;
mod util;

use coremidi::{PacketBuffer, VirtualSource};
pub use step_sequencer_client::SSCoreAudioClient;
use util::nanosecs_to_mach_ticks;

use crate::SSResult;

use super::adapter::MIDIAdapter;

trait Receivable {
    fn received(&self, packets: &PacketBuffer) -> Result<(), coreaudio::sys::OSStatus>;
}

impl Receivable for VirtualSource {
    fn received(&self, packets: &PacketBuffer) -> Result<(), coreaudio::sys::OSStatus> {
        self.received(packets)
    }
}

pub struct CoreAudioMIDIAdapter<'a> {
    virtual_source: &'a dyn Receivable,
    buffer_size: usize,
    sample_rate: u64,
    nanosecs_on_play: u64,
    last_n_frames: usize,
}

impl<'a> CoreAudioMIDIAdapter<'a> {
    fn n_frames(&self) -> usize {
        self.buffer_size
    }

    fn mach_ticks_from_offset(&self, offset: usize) -> u64 {
        let total_frames = self.last_n_frames + offset + self.buffer_size;
        let frame_latency_nanos = total_frames * 1_000_000_000 / self.sample_rate as usize;
        return self.nanosecs_on_play + nanosecs_to_mach_ticks(frame_latency_nanos as u64);
    }
}
impl<'a> MIDIAdapter for CoreAudioMIDIAdapter<'a> {
    fn write(&mut self, offset: usize, event: crate::midi::ChannelVoiceEvent) -> SSResult<()> {
        let data = event.to_data()?;
        let timestamp = self.mach_ticks_from_offset(offset);
        let packet_buffer = PacketBuffer::new(timestamp, &data);
        Ok(self.virtual_source.received(&packet_buffer)?)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use coremidi::PacketBuffer;

    use crate::audio::coreaudio::util::nanosecs_to_mach_ticks;

    use super::{CoreAudioMIDIAdapter, Receivable};

    struct MockReceivable {
        received: RefCell<Vec<usize>>,
    }

    impl Receivable for MockReceivable {
        fn received(&self, packets: &PacketBuffer) -> Result<(), coreaudio::sys::OSStatus> {
            self.received.borrow_mut().push(packets.len());
            Ok(())
        }
    }
    #[test]
    pub fn test_offset_zero_latency_of_buffer_size() {
        let virtual_source = MockReceivable {
            received: RefCell::new(Vec::new()),
        };
        let adapter = CoreAudioMIDIAdapter {
            virtual_source: &virtual_source,
            buffer_size: 1024,
            sample_rate: 44100,
            nanosecs_on_play: 1234567890,
            last_n_frames: 11500,
        };
        let ticks = adapter.mach_ticks_from_offset(250);
        assert_eq!(
            ticks,
            1234567890 + nanosecs_to_mach_ticks((11500 + 1024 + 250) * 1_000_000_000 / 44100)
        );
    }
}
