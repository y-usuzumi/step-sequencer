use crate::audio::Command;
use crate::beatmaker::BeatMaker;
use crate::midi::ChannelVoiceEvent;
use crate::project::Project;
use crate::{audio::SSClient, SSResult};

use coreaudio::audio_unit::render_callback::{self, data};
use coreaudio::audio_unit::{AudioUnit, IOType, SampleFormat};
use coremidi::{Client, PacketBuffer};
use log::{debug, info};
use std::f64::consts::PI;
use std::rc::Rc;
use std::sync::mpsc::Sender;
use std::thread;

pub struct SSCoreAudioClient {
    beatmaker: Rc<BeatMaker>,
    stop_signal_sender: Option<Sender<()>>,
}

impl SSCoreAudioClient {
    pub fn new(beatmaker: Rc<BeatMaker>) -> Self {
        Self {
            beatmaker,
            stop_signal_sender: None,
        }
    }
}

impl SSClient for SSCoreAudioClient {
    fn start(&self) -> SSResult<()> {
        info!("Running midi client");
        let beatmaker_subscription = self.beatmaker.subscribe();
        thread::spawn(move || -> SSResult<()> {
            let client = Client::new("Yukio's Step Sequencer MIDI").unwrap();
            let source = client.virtual_source("source").unwrap();
            // coreaudio_example_sinewave()?;
            for event in beatmaker_subscription.iter() {
                debug!(
                    "BeatMaker: subscription ID: {:?}",
                    beatmaker_subscription.id()
                );
                debug!("BeatMaker: Received event from: {:?}", event);
                let data = event.to_data()?;
                debug!("BeatMaker: MIDI data: {:?}", data);
                let time = match event {
                    ChannelVoiceEvent::NoteOff { .. } => 1,
                    _ => 0,
                };
                let packet_buffer = PacketBuffer::new(time, &data);
                source.received(&packet_buffer).unwrap();
            }
            Ok(())
        });
        // thread::sleep(Duration::from_secs(100));
        info!("SSCoreAudioClient started");
        Ok(())
    }

    fn stop(&self) -> SSResult<()> {
        if let Some(ref stop_signal_sender) = self.stop_signal_sender {
            stop_signal_sender.send(());
        }
        Ok(())
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
