use jack::{MidiWriter, RawMidi};

use crate::audio::midi_adapter::MIDIAdapter;

pub struct JackMIDIAdapter<'a> {
    midi_writer: MidiWriter<'a>,
}

impl<'a> JackMIDIAdapter<'a> {
    pub fn new(midi_writer: MidiWriter<'a>) -> Self {
        Self { midi_writer }
    }
}

impl<'a> MIDIAdapter for JackMIDIAdapter<'a> {
    fn write(
        &mut self,
        offset: usize,
        event: crate::midi::ChannelVoiceEvent,
    ) -> crate::SSResult<()> {
        let data = event.to_data()?;
        let raw_midi = RawMidi {
            time: offset as u32,
            bytes: &data,
        };
        let result = self.midi_writer.write(&raw_midi)?;
        Ok(result)
    }
}
