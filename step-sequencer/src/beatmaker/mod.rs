pub mod beat_sorter;
pub mod beat_time;
pub mod pattern;

use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard},
    thread,
};

use beat_sorter::BeatSorter;
use beat_time::BeatTime;
use crossbeam::{
    channel::{bounded, unbounded, Receiver, Sender},
    select,
};
use log::{debug, info};

use crate::{
    drum_track::Beat,
    id::{AutoIncrementId, AutoIncrementIdGen},
    midi::ChannelVoiceEvent,
    project::{Project, TrackMap, F},
    timeline::{TimelineEvent, TimelineSubscription},
};

fn send_beat(subscribers: &RwLockReadGuard<BeatMakerSubscriberMap>, beat: &Beat) {
    debug!("BeatMaker: Sending events");
    for sender in subscribers.values() {
        let _ = sender.send(ChannelVoiceEvent::NoteOn {
            channel: beat.channel,
            key: beat.note.into(),
            velocity: beat.velocity,
        });
        let _ = sender.send(ChannelVoiceEvent::NoteOff {
            channel: beat.channel,
            key: beat.note.into(),
            velocity: beat.velocity,
        });
    }
}

pub enum BeatSignal {
    Beat(BeatTime),
    Pause,
    Stop,
}

enum InternalSignal {
    ResetSorter,
}

type BeatMakerSubscriberMap = HashMap<AutoIncrementId, Sender<ChannelVoiceEvent>>;
type SignalSubscriberMap = Vec<Sender<BeatSignal>>;

/// BeatMaker sends walks along the timeline and send out beats of every track
/// as MIDI notes.
/// Internally, it maintains a search tree of the next upcoming notes of each track.
/// This ensures all notes are sent out in the correct order.
pub struct BeatMaker {
    subscribers: Arc<RwLock<BeatMakerSubscriberMap>>,
    signal_subscribers: Arc<RwLock<SignalSubscriberMap>>,
    idgen: RefCell<AutoIncrementIdGen>,
    internal_signal: (Sender<InternalSignal>, Receiver<InternalSignal>),
}

impl BeatMaker {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn subscribe(&self) -> BeatMakerSubscription {
        let next_id = self.idgen.borrow_mut().next();
        let (sender, receiver) = unbounded();
        let mut subscriber_map = self.subscribers.write().unwrap();
        subscriber_map.insert(next_id, sender);

        BeatMakerSubscription {
            id: next_id,
            receiver,
            subscribers: self.subscribers.clone(),
        }
    }

    pub fn subscribe_signals(&self) -> Receiver<BeatSignal> {
        let mut signal_subscribers = self.signal_subscribers.write().unwrap();
        let (sender, receiver) = unbounded();
        signal_subscribers.push(sender);
        return receiver;
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
        let subscribers = self.subscribers.clone();
        let signal_subscribers = self.signal_subscribers.clone();
        let internal_signal_receiver = self.internal_signal.1.clone();
        thread::spawn(move || {
            info!("BeatMaker started");
            let interval = timeline_subscription.interval;
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
                                let subscribers = subscribers.read().unwrap();
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
                                    let signal_subscribers = signal_subscribers.read().unwrap();
                                    for signal_subscriber in signal_subscribers.iter() {
                                        signal_subscriber.send(BeatSignal::Beat(beat_time));
                                    }
                                }
                                let beat_time_incr =
                                    F::new(tempo as u64 * interval.as_millis() as u64, 60_000u64);
                                current_beat_time = current_beat_time.add_fraction(beat_time_incr);
                            }
                            TimelineEvent::Pause => {
                                let signal_subscribers = signal_subscribers.read().unwrap();
                                for signal_subscriber in signal_subscribers.iter() {
                                    signal_subscriber.send(BeatSignal::Pause);
                                }
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
                                let signal_subscribers = signal_subscribers.read().unwrap();
                                for signal_subscriber in signal_subscribers.iter() {
                                    signal_subscriber.send(BeatSignal::Stop);
                                }
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
            subscribers: Default::default(),
            signal_subscribers: Default::default(),
            idgen: Default::default(),
            internal_signal: bounded(1),
        }
    }
}

pub struct BeatMakerAsyncHandle;

pub struct BeatMakerSubscription {
    pub id: AutoIncrementId,
    pub receiver: Receiver<ChannelVoiceEvent>,
    pub subscribers: Arc<RwLock<BeatMakerSubscriberMap>>,
}

impl Drop for BeatMakerSubscription {
    fn drop(&mut self) {
        let mut subscriber_map = self.subscribers.write().unwrap();
        subscriber_map.remove(&self.id);
    }
}
