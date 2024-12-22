use std::{thread, time::Duration};

use coremidi::{Client, PacketBuffer};

use crate::SSResult;

pub fn create_midi_client() -> SSResult<Client> {
    let client = Client::new("Yukio's Step Sequencer MIDI").unwrap();
    let source = client.virtual_source("source").unwrap();

    let note_on = create_note_on(0, 64, 127);
    let note_off = create_note_off(0, 64, 127);

    for i in 0..10 {
        println!("[{}] Sending note...", i);

        source.received(&note_on).unwrap();

        thread::sleep(Duration::from_millis(1000));

        source.received(&note_off).unwrap();
    }

    Ok(client)
}

fn create_note_on(channel: u8, note: u8, velocity: u8) -> PacketBuffer {
    let data = &[0x90 | (channel & 0x0f), note & 0x7f, velocity & 0x7f];
    PacketBuffer::new(0, data)
}

fn create_note_off(channel: u8, note: u8, velocity: u8) -> PacketBuffer {
    let data = &[0x80 | (channel & 0x0f), note & 0x7f, velocity & 0x7f];
    PacketBuffer::new(0, data)
}
