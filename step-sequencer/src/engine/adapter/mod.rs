use crate::{midi::ChannelVoiceEvent, SSResult};

pub trait MIDIAdapter {
    fn write(&mut self, offset: usize, event: ChannelVoiceEvent) -> SSResult<()>;
}
