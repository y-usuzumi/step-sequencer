use crate::audio::async_processor::{AsyncProcessor, AsyncProcessorState};
use crate::audio::SSClient2;
use crate::beatmaker::beat_sorter::BeatSorter;
use crate::beatmaker::beat_time::BeatTime;
use crate::beatmaker::BeatMakerSubscriptionModel;
use crate::project::ProjectSettings;
use crate::SSResult;

use std::sync::{Arc, RwLock};

use crate::audio::jack::midi_adapter::JackMIDIAdapter;

struct ClientState {
    current_beat_time: BeatTime,
    is_playing: bool,
}

impl ClientState {
    fn new() -> Self {
        Self {
            current_beat_time: BeatTime::zero(),
            is_playing: false,
        }
    }
}

pub struct SSJackClient2 {
    beat_sorter: Arc<RwLock<BeatSorter>>,
    project_settings: Arc<RwLock<ProjectSettings>>,
    beat_subscription_model: BeatMakerSubscriptionModel,
    client_state: Arc<RwLock<ClientState>>,
}

impl SSJackClient2 {
    pub fn new(
        beat_sorter: Arc<RwLock<BeatSorter>>,
        project_settings: Arc<RwLock<ProjectSettings>>,
    ) -> Self {
        // 1. Create client
        let (client, _status) =
            jack::Client::new("Yukio's Step Sequencer", jack::ClientOptions::default()).unwrap();

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
        let mut async_processor =
            AsyncProcessor::new(beat_sorter.clone(), project_settings.clone());
        let process_callback = move |state: &mut AsyncProcessorState,
                                     client: &jack::Client,
                                     process_scope: &jack::ProcessScope|
              -> jack::Control {
            state.last_n_frames = process_scope.last_frame_time() as u64;
            let out_a_p = out_a.as_mut_slice(process_scope);
            let out_b_p = out_b.as_mut_slice(process_scope);
            let in_a_p = in_a.as_slice(process_scope);
            let in_b_p = in_b.as_slice(process_scope);
            out_a_p.clone_from_slice(in_a_p);
            out_b_p.clone_from_slice(in_b_p);
            let midi_writer = out_midi.writer(process_scope);
            let mut midi_adapter = JackMIDIAdapter::new(midi_writer);
            async_processor.on_process_cycle(&mut midi_adapter, state);

            jack::Control::Continue
        };
        let process = jack::contrib::ClosureProcessHandler::with_state(
            AsyncProcessorState {
                sample_rate: client.sample_rate() as u64,
                buffer_size: client.buffer_size() as u64,
                last_n_frames: 0,
                current_played_ticks: 0,
            },
            process_callback,
            move |_, _, _| jack::Control::Continue,
        );
        // 3. Activate the client, which starts the processing.
        // Must bind it to a variable, otherwise the client is deactivated on drop immediately!
        let active_client = client.activate_async((), process).unwrap();

        Self {
            beat_sorter,
            project_settings,
            beat_subscription_model: BeatMakerSubscriptionModel::default(),
            client_state: Arc::new(RwLock::new(ClientState::new())),
        }
    }
}

impl SSClient2 for SSJackClient2 {
    fn start(&mut self) -> SSResult<()> {
        let mut client_state = self.client_state.write().unwrap();
        client_state.is_playing = true;
        Ok(())
    }

    fn pause(&mut self) -> SSResult<()> {
        let mut client_state = self.client_state.write().unwrap();
        client_state.is_playing = false;
        Ok(())
    }

    fn stop(&mut self) -> SSResult<()> {
        let mut client_state = self.client_state.write().unwrap();
        client_state.is_playing = true;
        client_state.current_beat_time = BeatTime::zero();
        Ok(())
    }
}
