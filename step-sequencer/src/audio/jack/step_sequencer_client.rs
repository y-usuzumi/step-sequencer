use std::{
    sync::mpsc::{self, Sender},
    thread,
    time::Duration,
};

use crate::{audio::Command, error::SSError, project::Project};
use jack::{contrib::ClosureProcessHandler, AsyncClient, Frames, RawMidi};
use log::{debug, error, info};

use crate::{
    audio::SSClient,
    beatmaker::{BeatMaker, BeatMakerSubscription},
    midi::ChannelVoiceEvent,
    SSResult,
};

pub struct SSJackClient<'a> {
    beatmaker: BeatMaker<'a>,
    project: &'a Project,
    stop_signal_sender: Option<Sender<()>>,
}

impl<'a> SSJackClient<'a> {
    pub fn new(beatmaker: BeatMaker<'a>, project: &'a Project) -> Self {
        Self {
            beatmaker,
            project,
            stop_signal_sender: None,
        }
    }

    fn create_ss_jack_client(&mut self) {
        let (stop_signal_sender, stop_signal_receiver) = mpsc::channel();
        self.stop_signal_sender = Some(stop_signal_sender);
        let beatmaker_subscription = self.beatmaker.subscribe();
        self.beatmaker.start(&self.project);
        thread::spawn(move || -> SSResult<()> {
            // 1. Create client
            let (client, _status) =
                jack::Client::new("Yukio's Step Sequencer", jack::ClientOptions::default())
                    .unwrap();

            // 2. Register ports. They will be used in a callback that will be
            // called when new data is available.
            let in_a: jack::Port<jack::AudioIn> = client
                .register_port("in_audio_l", jack::AudioIn::default())
                .unwrap();
            let in_b: jack::Port<jack::AudioIn> = client
                .register_port("in_audio_r", jack::AudioIn::default())
                .unwrap();
            let mut out_a: jack::Port<jack::AudioOut> = client
                .register_port("out_audio_l", jack::AudioOut::default())
                .unwrap();
            let mut out_b: jack::Port<jack::AudioOut> = client
                .register_port("out_audio_r", jack::AudioOut::default())
                .unwrap();
            let mut out_sinewave: jack::Port<jack::AudioOut> = client
                .register_port("out_sinewave", jack::AudioOut::default())
                .unwrap();
            let mut out_midi: jack::Port<jack::MidiOut> = client
                .register_port("out_midi", jack::MidiOut::default())
                .unwrap();
            // thread::sleep(Duration::from_secs(100));
            let process_callback = move |state: &mut TestState,
                                         client: &jack::Client,
                                         process_scope: &jack::ProcessScope|
                  -> jack::Control {
                let out_a_p = out_a.as_mut_slice(process_scope);
                let out_b_p = out_b.as_mut_slice(process_scope);
                let in_a_p = in_a.as_slice(process_scope);
                let in_b_p = in_b.as_slice(process_scope);
                out_a_p.clone_from_slice(in_a_p);
                out_b_p.clone_from_slice(in_b_p);

                // Sine wave test
                process_sine_wave(client, &mut out_sinewave, process_scope);

                // Midi test
                // let _ = process_midi(state, client, &mut out_midi, process_scope);
                let _ = process_beatmaker(
                    &beatmaker_subscription,
                    state,
                    client,
                    &mut out_midi,
                    process_scope,
                );

                let frame_time = process_scope.last_frame_time();
                let frames = process_scope.n_frames();
                let sample_rate = client.sample_rate() as u32;
                let seconds = (frame_time + frames) / sample_rate;
                if seconds > state.last_event_midi_seconds {
                    state.last_event_midi_seconds = seconds;
                }

                jack::Control::Continue
            };
            let process = jack::contrib::ClosureProcessHandler::with_state(
                TestState {
                    last_event_midi_seconds: 0,
                },
                process_callback,
                move |_, _, _| jack::Control::Continue,
            );
            // 3. Activate the client, which starts the processing.
            // Must bind it to a variable, otherwise the client is deactivated on drop immediately!
            let active_client = client.activate_async((), process).unwrap();
            if let Ok(_) = stop_signal_receiver.recv() {
                Ok(())
            } else {
                Err(crate::error::SSError::Unknown(
                    "stop_signal_sender disconnected while receiving from stop_signal_receiver"
                        .to_string(),
                ))
            }
        });
    }
}

impl<'a> SSClient for SSJackClient<'a> {
    fn start(&mut self) -> SSResult<()> {
        self.create_ss_jack_client();
        info!("SSJackClient started");
        Ok(())
    }

    fn stop(&mut self) -> SSResult<()> {
        // 5. Not needed as the async client will cease processing on `drop`.
        // if let Err(err) = active_client.deactivate() {
        //     error!("JACK exited with error: {err}");
        // }
        if let Some(ref stop_signal_sender) = self.stop_signal_sender {
            stop_signal_sender.send(());
        }
        Ok(())
    }
}

struct TestState {
    last_event_midi_seconds: u32,
}

fn frame_to_time(frames: Frames, sample_rate: usize) -> f64 {
    frames as f64 / sample_rate as f64
}

fn process_sine_wave(
    client: &jack::Client,
    port: &mut jack::Port<jack::AudioOut>,
    process_scope: &jack::ProcessScope,
) {
    let a440 = 220.;
    let out_sinewave_p = port.as_mut_slice(process_scope);
    let last_frame_time = process_scope.last_frame_time();
    let sample_rate = client.sample_rate();
    for (idx, v) in out_sinewave_p.iter_mut().enumerate() {
        let time_elapsed = frame_to_time(last_frame_time + idx as u32, sample_rate);
        let x = a440 * time_elapsed * 2.0 * std::f64::consts::PI;
        let y = x.sin();
        *v = y as f32;
    }
}

fn process_midi(
    state: &mut TestState,
    client: &jack::Client,
    port: &mut jack::Port<jack::MidiOut>,
    process_scope: &jack::ProcessScope,
) -> SSResult<()> {
    let frame_time = process_scope.last_frame_time();
    let frames = process_scope.n_frames();
    let sample_rate = client.sample_rate() as u32;
    info!("Sample rate: {:?}", sample_rate);
    let mut midi_writer = port.writer(process_scope);
    let seconds = (frame_time + frames) / sample_rate;
    if seconds > state.last_event_midi_seconds {
        debug!("Frame time: {:?}, frames: {:?}", frame_time, frames);
        debug!("Seconds: {:?}", seconds);
        let message = if seconds % 2 != 0 {
            ChannelVoiceEvent::NoteOn {
                channel: 0,
                key: 64,
                velocity: 64,
            }
        } else {
            ChannelVoiceEvent::NoteOff {
                channel: 0,
                key: 64,
                velocity: 64,
            }
        };
        let data = message.to_data()?;
        debug!("MIDI data: {:?}", data);
        let raw_midi = RawMidi {
            time: 1,
            bytes: &data,
        };
        midi_writer.write(&raw_midi)?;
    }

    Ok(())
}

fn process_beatmaker(
    subscription: &BeatMakerSubscription,
    state: &mut TestState,
    client: &jack::Client,
    port: &mut jack::Port<jack::MidiOut>,
    process_scope: &jack::ProcessScope,
) -> SSResult<()> {
    // TODO: Can NOT use while loop to process all messages in the channel.
    // Find out why.
    if let Ok(event) = &subscription.try_recv() {
        debug!("BeatMaker: subscription ID: {:?}", subscription.id());
        debug!("BeatMaker: Received event from: {:?}", event);
        let data = event.to_data()?;
        debug!("BeatMaker: MIDI data: {:?}", data);
        let time = match event {
            ChannelVoiceEvent::NoteOff { .. } => 1,
            _ => 0,
        };
        let raw_midi = RawMidi { time, bytes: &data };
        let mut midi_writer = port.writer(process_scope);
        midi_writer.write(&raw_midi)?;
    }
    Ok(())
}