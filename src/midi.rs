use std::io::{self, Write};

use self::ChannelVoiceEvent::*;

pub type Key = u8; // TODO:
pub type Channel = u8;
pub type Velocity = u8;

pub enum ChannelVoiceEvent {
    NoteOn {
        channel: Channel,
        key: Key,
        velocity: Velocity,
    },
    NoteOff {
        channel: Channel,
        key: Key,
        velocity: Velocity,
    },
}

pub struct Message {
    cmd: u8,
    channel: u8,
    data_bytes: Vec<u8>,
}

impl ChannelVoiceEvent {
    pub fn to_message(&self) -> Message {
        let (cmd, channel, data_bytes) = match *self {
            NoteOn {
                channel,
                key,
                velocity,
            } => (0b1000, channel, &[key, velocity]),
            NoteOff {
                channel,
                key,
                velocity,
            } => (0b1001, channel, &[key, velocity]),
        };
        Message {
            cmd,
            channel,
            data_bytes: data_bytes.to_vec(),
        }
    }

    pub fn to_data(&self) -> io::Result<Vec<u8>> {
        let msg = self.to_message();
        msg.to_data()
    }

    pub fn write_to_buffer(&self, buf: &mut [u8]) -> io::Result<()> {
        let msg = self.to_message();
        msg.write_to_buffer(buf)
    }
}

impl Message {
    fn to_data(&self) -> io::Result<Vec<u8>> {
        let mut result = vec![0; self.data_bytes.len() + 1];
        self.write_to_buffer(&mut result)?;
        Ok(result)
    }

    // Write is impl'd on &mut [u8], so write_all is actually called
    // on &mut &mut buf. That's why buf itself needs to be declared `mut`.
    fn write_to_buffer(&self, mut buf: &mut [u8]) -> io::Result<()> {
        let data = [&[self.cmd << 4 | self.channel] as &[u8], &self.data_bytes].concat();
        (&mut buf).write_all(&data)
    }
}

#[cfg(test)]
mod tests {
    use super::ChannelVoiceEvent;

    #[test]
    fn test_write_note_on() {
        let event = ChannelVoiceEvent::NoteOn {
            channel: 1,
            key: 10,
            velocity: 42,
        };
        let data = event.to_data().unwrap();
        assert_eq!(data, &[0b10000001, 0b00001010, 0b00101010]);
    }
}
