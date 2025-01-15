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
    drum_track::Beat,
    midi::ChannelVoiceEvent,
    models::channel_subscription::{ChannelEventSubscription, ChannelEventSubscriptionModel},
    project::{Project, TrackMap, F},
    timeline::{TimelineEvent, TimelineSubscription},
};

enum InternalSignal {
    ResetSorter,
}

#[derive(Clone, PartialEq, serde::Serialize)]
pub enum BeatMakerEvent {
    Pause,                             // Timeline pause
    Stop,                              // Timeline reset
    CompleteTick(u64),                 // Passthru of timeline ticks
    Step(u64),                         // Global steps elapsed
    Beat(BeatTime),                    // Track beat
    MIDIEvent(u64, ChannelVoiceEvent), // MIDI event to be processed by audio server (frames since play start, MIDI event)
}

type BeatMakerSubscriptionModel = ChannelEventSubscriptionModel<BeatMakerEvent>;
type BeatMakerSubscription = ChannelEventSubscription<BeatMakerEvent>;

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

    fn populate_beat_sorter_for_beat_time(
        beat_sorter: &mut BeatSorter,
        current_beat_time: BeatTime,
        tracks: &TrackMap,
    ) {
        for (track_id, track) in tracks.iter() {
            let tempo_scale = track.get_tempo_scale();
            let track_beat_time = current_beat_time.stretch(tempo_scale).ceil();
            beat_sorter.push(
                *track_id,
                track_beat_time.compress(tempo_scale),
                track.get_as_beats(track_beat_time.integral()).flatten(),
            );
        }
    }

    pub fn start(&self, project: &Project, timeline_subscription: TimelineSubscription) {
        let project_settings = project.project_settings();
        let tracks = project.tracks();
        let internal_signal_receiver = self.internal_signal.1.clone();
        let subscriber_map = self.subscription_model.subscriber_map().clone();
        thread::spawn(move || {
            info!("BeatMaker started");
            let mut beat_sorter = BeatSorter::new();
            let mut current_beat_time = BeatTime::zero();
            loop {
                select! {
                    recv(internal_signal_receiver) -> signal => {
                        match signal {
                            Ok(InternalSignal::ResetSorter) => {
                                beat_sorter.reset();
                                Self::populate_beat_sorter_for_beat_time(&mut beat_sorter, current_beat_time, &tracks.read().unwrap());
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
                                if tick == 0 {
                                    // Start playing
                                    Self::populate_beat_sorter_for_beat_time(
                                        &mut beat_sorter,
                                        current_beat_time,
                                        &tracks.read().unwrap(),
                                    );
                                }
                                let project_settings = project_settings.read().unwrap();
                                *project_settings.current_beat_time.write().unwrap() = current_beat_time;
                                let tempo = project_settings.tempo;
                                drop(project_settings);
                                while beat_sorter.top().is_some() {
                                    if *beat_sorter.top().unwrap().0 > current_beat_time {
                                        break;
                                    }
                                    let (beat_time, beats) = beat_sorter.pop().unwrap();
                                    let tracks = tracks.read().unwrap();
                                    for (track_id, beats_in_track) in beats {
                                        if let Some(bs) = beats_in_track {
                                            bs.iter().for_each(|b| send_beat(&subscribers, &b));

                                        }
                                        if let Some(track) = tracks.get(&track_id) {
                                            let tempo_scale = track.get_tempo_scale();
                                            let track_beat_time =
                                                current_beat_time.stretch(tempo_scale);
                                            let next_track_beat_time = track_beat_time.add_integral(1).floor();
                                            let next_beat_idx = if track.is_empty() {
                                                0
                                            } else {
                                                next_track_beat_time.integral() % track.len()
                                            };
                                            let next_beat = track
                                                .get_as_beats(next_beat_idx)
                                                .flatten();
                                            beat_sorter.push(track_id,
                                                next_track_beat_time.compress(tempo_scale), next_beat);
                                        }
                                    }
                                    ChannelEventSubscriptionModel::send_all(&subscriber_map, BeatMakerEvent::Beat(beat_time));
                                }
                                let beat_time_incr =
                                    F::new(tempo as u64 * consts::TIMELINE_TICK_DURATION.as_millis() as u64, 60_000u64);
                                current_beat_time = current_beat_time.add_fraction(beat_time_incr);
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
