pub mod beat_sorter;
pub mod beat_time;
pub mod pattern;

use std::thread;

use beat_sorter::BeatSorter;
use beat_time::BeatTime;
use crossbeam::{
    channel::{bounded, Receiver, Sender},
    select,
};
use log::{debug, info};

use crate::{
    consts,
    midi::ChannelVoiceEvent,
    models::channel_subscription::{ChannelEventSubscription, ChannelEventSubscriptionModel},
    project::{Project, F},
    timeline::{TimelineEvent, TimelineSubscription},
};

enum InternalSignal {
    ResetSorter,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub enum BeatMakerEvent {
    Pause,                        // Timeline pause
    Stop,                         // Timeline reset
    Tick(u64),                    // Passthru of timeline ticks
    Step(u64),                    // Global steps elapsed
    Beat(BeatTime),               // Track beat
    MIDIEvent(ChannelVoiceEvent), // MIDI event to be processed by audio server
}

pub type BeatMakerSubscriptionModel = ChannelEventSubscriptionModel<BeatMakerEvent>;
pub type BeatMakerSubscription = ChannelEventSubscription<BeatMakerEvent>;

/// BeatMaker sends walks along the timeline and send out beats of every track
/// as MIDI notes.
/// Internally, it maintains a search tree of the next upcoming notes of each track.
/// This ensures all notes are sent out in the correct order.
pub struct BeatMaker {
    subscription_model: BeatMakerSubscriptionModel,
    internal_signal: (Sender<InternalSignal>, Receiver<InternalSignal>),
}

impl BeatMaker {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn subscribe(&self) -> BeatMakerSubscription {
        self.subscription_model.subscribe()
    }

    pub fn start(&self, project: &Project, timeline_subscription: TimelineSubscription) {
        let project_settings = project.project_settings();
        let tracks = project.tracks();
        let internal_signal_receiver = self.internal_signal.1.clone();
        let subscriber_map = self.subscription_model.subscriber_map().clone();
        thread::spawn(move || {
            info!("BeatMaker started");
            let mut beat_sorter = BeatSorter::with_tracks(tracks);
            let mut current_beat_time = BeatTime::zero();
            loop {
                select! {
                    recv(internal_signal_receiver) -> signal => {
                        match signal {
                            Ok(InternalSignal::ResetSorter) => {
                                beat_sorter.reset();
                            }
                            Err(err) => {
                                info!("Internal signal sender: {}. Exiting BeatMaker thread.", err);
                                break;
                            }
                        }

                    }
                    recv(timeline_subscription.receiver) -> tick => {
                        match tick.unwrap() {
                            TimelineEvent::Tick(tick) => {
                                BeatMakerSubscriptionModel::send_all(&subscriber_map, BeatMakerEvent::Tick(tick));
                                let tempo = {
                                    let project_settings = project_settings.read().unwrap();
                                    *project_settings.current_beat_time.write().unwrap() = current_beat_time;
                                    project_settings.tempo
                                };

                                let beat_time_per_tick =
                                    F::new(tempo as u64 * consts::TIMELINE_TICK_DURATION.as_millis() as u64, 60_000u64);
                                let beat_time = BeatTime::new(beat_time_per_tick * tick);
                                let beats = beat_sorter.advance(beat_time);
                                for (_beat_time, beats) in beats.iter() {
                                    BeatMakerSubscriptionModel::send_all(&subscriber_map, BeatMakerEvent::Beat(beat_time));
                                    for (_id, beat) in beats {
                                        BeatMakerSubscriptionModel::send_all(&subscriber_map, BeatMakerEvent::MIDIEvent(*beat));
                                    }

                                }

                                current_beat_time = beat_time;
                            }
                            TimelineEvent::Pause => {
                                BeatMakerSubscriptionModel::send_all(&subscriber_map, BeatMakerEvent::Pause);
                            }
                            TimelineEvent::Stop => {
                                *project_settings
                                    .read()
                                    .unwrap()
                                    .current_beat_time
                                    .write()
                                    .unwrap() = BeatTime::zero();
                                current_beat_time = BeatTime::zero();
                                beat_sorter.reset();
                                BeatMakerSubscriptionModel::send_all(&subscriber_map, BeatMakerEvent::Stop);
                            }
                        }
                    }
                }
            }
        });
    }

    pub fn reload_beat_sorter(&self) {
        self.internal_signal.0.send(InternalSignal::ResetSorter);
    }
}

impl Default for BeatMaker {
    fn default() -> Self {
        Self {
            subscription_model: Default::default(),
            internal_signal: bounded(1),
        }
    }
}

pub struct BeatMakerAsyncHandle;
