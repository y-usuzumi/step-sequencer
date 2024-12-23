use std::{thread, time::Duration};

use coremidi::{Client, PacketBuffer};

use crate::{
    beatmaker::{pattern::BEAT_NOTE_MAP_GARAGEBAND, BeatMaker},
    midi::ChannelVoiceEvent,
    SSResult,
};

pub fn create_midi_client() -> SSResult<Client> {
    let client = Client::new("Yukio's Step Sequencer MIDI").unwrap();
    let source = client.virtual_source("source").unwrap();

    let mut beatmaker = BeatMaker::default();
    let _ = beatmaker.start(BEAT_NOTE_MAP_GARAGEBAND);
    let beatmaker_subscription = beatmaker.subscribe();
    for event in beatmaker_subscription.iter() {
        println!(
            "BeatMaker: subscription ID: {:?}",
            beatmaker_subscription.id()
        );
        println!("BeatMaker: Received event from: {:?}", event);
        let data = event.to_data()?;
        println!("BeatMaker: MIDI data: {:?}", data);
        let time = match event {
            ChannelVoiceEvent::NoteOff { .. } => 1,
            _ => 0,
        };
        let packet_buffer = PacketBuffer::new(time, &data);
        source.received(&packet_buffer).unwrap();
    }

    Ok(client)
}
