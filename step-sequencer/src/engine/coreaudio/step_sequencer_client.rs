use crate::engine::coreaudio::util::{current_mach_ticks_since_boot, nanosecs_to_mach_ticks};
use crate::beatmaker::{BeatMaker, BeatMakerEvent, BeatMakerSubscription};
use crate::error::SSError;
use crate::midi::ChannelVoiceEvent;
use crate::project::Project;
use crate::{engine::SSClient, SSResult};

use coreaudio::audio_unit::render_callback::{self, data};
use coreaudio::audio_unit::{AudioUnit, IOType, SampleFormat};
use coremidi::{Client, PacketBuffer};
use crossbeam::channel::{bounded, Sender};
use crossbeam::select;
use log::{debug, info};
use std::f64::consts::PI;
use std::sync::Arc;
use std::thread::{self, JoinHandle};

pub struct SSCoreAudioClient {
    beatmaker_subscription: Arc<BeatMakerSubscription>,
    stop_signal_sender: Option<Sender<()>>,
    processor_thread: Option<JoinHandle<SSResult<()>>>,
}

impl SSCoreAudioClient {
    pub fn new(beatmaker_subscription: BeatMakerSubscription) -> Self {
        Self {
            beatmaker_subscription: Arc::new(beatmaker_subscription),
            stop_signal_sender: None,
            processor_thread: None,
        }
    }
}

impl SSClient for SSCoreAudioClient {
    fn start(&mut self) -> SSResult<()> {
        if self.processor_thread.is_some() {
            return Err(SSError::Unknown("SSClient is started already".to_string()));
        }
        let beatmaker_subscription = self.beatmaker_subscription.clone();
        let (stop_signal_sender, stop_signal_receiver) = bounded(1);
        self.stop_signal_sender = Some(stop_signal_sender);
        let join_handle = thread::spawn(move || -> SSResult<()> {
            info!("Running midi client");
            let client = Client::new("Yukio's Step Sequencer MIDI").unwrap();
            let source = client.virtual_source("source").unwrap();
            loop {
                select! {
                    recv(beatmaker_subscription.receiver) -> event => {
                        let event = event?;
                        match event {
                            BeatMakerEvent::MIDIEvent(evt) => {
                                let data = evt.to_data()?;
                                debug!("BeatMaker: MIDI data: {:?}", data);
                                let packet_buffer = PacketBuffer::new(0, &data);
                                source.received(&packet_buffer).unwrap();
                            },
                            _ => {}
                        }
                    }
                    recv(stop_signal_receiver) -> stop_signal => {
                        let _ = stop_signal?;
                        break;
                    }
                }
            }
            Ok(())
        });
        self.processor_thread = Some(join_handle);
        // thread::sleep(Duration::from_secs(100));
        info!("SSCoreAudioClient started");
        Ok(())
    }

    fn stop(&mut self) -> SSResult<()> {
        match self.processor_thread {
            Some(_) => {
                self.stop_signal_sender.take().unwrap().send(());
                self.processor_thread.take().unwrap().join().unwrap();
                Ok(())
            }
            None => Ok(()),
        }
    }
}

struct SineWaveGenerator {
    time: f64,
    /// generated frequency in Hz
    freq: f64,
    /// magnitude of generated signal
    volume: f64,
}

impl SineWaveGenerator {
    fn new(freq: f64, volume: f64) -> Self {
        SineWaveGenerator {
            time: 0.,
            freq,
            volume,
        }
    }
}

impl Iterator for SineWaveGenerator {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        self.time += 1. / 44_100.;
        let output = ((self.freq * self.time * PI * 2.).sin() * self.volume) as f32;
        Some(output)
    }
}

fn coreaudio_example_sinewave() -> Result<(), coreaudio::Error> {
    let frequency_hz = 440.;
    let volume = 0.15;
    let mut samples = SineWaveGenerator::new(frequency_hz, volume);

    // Construct an Output audio unit that delivers audio to the default output device.
    let mut audio_unit = AudioUnit::new(IOType::DefaultOutput)?;

    // Read the input format. This is counterintuitive, but it's the format used when sending
    // audio data to the AudioUnit representing the output device. This is separate from the
    // format the AudioUnit later uses to send the data to the hardware device.
    let stream_format = audio_unit.output_stream_format()?;
    debug!("{:#?}", &stream_format);

    // For this example, our sine wave expects `f32` data.
    assert!(SampleFormat::F32 == stream_format.sample_format);

    type Args = render_callback::Args<data::NonInterleaved<f32>>;
    audio_unit.set_render_callback(move |args| {
        let Args {
            num_frames,
            mut data,
            ..
        } = args;
        for i in 0..num_frames {
            let sample = samples.next().unwrap();
            for channel in data.channels_mut() {
                channel[i] = sample;
            }
        }
        Ok(())
    })?;
    audio_unit.start()?;

    std::thread::sleep(std::time::Duration::from_millis(3000));

    Ok(())
}
