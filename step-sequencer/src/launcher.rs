use std::{rc::Rc, sync::RwLockWriteGuard};

use crossbeam::channel::Receiver;
use log::{error, info};

use crate::{
    audio::{create_ss_client, Command, SSClient},
    beatmaker::{BeatMaker, BeatSignal},
    drum_track::DrumTrack,
    error::SSError,
    project::{Project, TrackMap},
    timeline::Timeline,
    SSResult,
};

/// SSLauncher orchestrates the audio client, timeline, beatmaker.
/// It handles the lifecycle of those components and estable communication
/// between them.
///
/// SSLauncher also handles global commands such as play, pause and track
/// operations.
pub struct SSLauncher {
    timeline: Timeline,
    beatmaker: BeatMaker,
    project: Rc<Project>,
    ss_client: Box<dyn SSClient>,
}

fn get_track<'a>(
    track_map: &'a mut RwLockWriteGuard<TrackMap>,
    track_idx: usize,
) -> Option<&'a mut DrumTrack> {
    let mut tracks = track_map.values_mut();
    for _ in 0..track_idx {
        tracks.next();
    }
    tracks.next()
}

impl SSLauncher {
    pub fn new() -> Self {
        let timeline = Timeline::default();
        let beatmaker = BeatMaker::new();
        let project = Rc::new(Project::new());
        let ss_client = create_ss_client(beatmaker.subscribe());
        Self {
            timeline,
            beatmaker,
            project,
            ss_client,
        }
    }

    pub fn start(&mut self) -> SSResult<()> {
        self.ss_client.start()?;
        self.beatmaker
            .start(&self.project, self.timeline.subscribe());
        Ok(())
    }

    pub fn stop(&mut self) -> SSResult<()> {
        self.timeline.stop();
        // self.beatmaker.stop();
        self.ss_client.stop()
    }

    pub fn project(&self) -> Rc<Project> {
        self.project.clone()
    }

    pub fn timeline(&self) -> &Timeline {
        &self.timeline
    }

    pub fn subscribe_beatmaker_signals(&self) -> Receiver<BeatSignal> {
        self.beatmaker.subscribe_signals()
    }

    pub fn send_command(&self, command: Command) -> SSResult<()> {
        match command {
            Command::ChangeTempo(tempo) => {
                info!("Global tempo -> {}", tempo);
                let project_settings = self.project.project_settings();
                project_settings.write().unwrap().tempo = tempo;
            }
            Command::ToggleBeat(track_idx, beat) => {
                let binding = self.project.tracks();
                let mut trackmap = binding.write().unwrap();
                let track = get_track(&mut trackmap, track_idx).ok_or(SSError::CommandError(
                    crate::error::CommandError::CommandExecutionError(
                        command,
                        format!("Track {} does not exist", track_idx),
                    ),
                ))?;
                info!("[🛤️ {}] Toggle beat @ {}", track_idx + 1, beat + 1);
                track.toggle_beat(beat);
            }
            Command::Resize(track_idx, size) => {
                let binding = self.project.tracks();
                let mut trackmap = binding.write().unwrap();
                let track = get_track(&mut trackmap, track_idx).ok_or(SSError::CommandError(
                    crate::error::CommandError::CommandExecutionError(
                        command,
                        format!("Track {} does not exist", track_idx),
                    ),
                ))?;
                info!("[🛤️ {}] Resize -> {}", track_idx + 1, size);
                track.resize(size);
            }
            Command::TempoScale(track_idx, scale) => {
                let binding = self.project.tracks();
                let mut trackmap = binding.write().unwrap();
                let track = get_track(&mut trackmap, track_idx).ok_or(SSError::CommandError(
                    crate::error::CommandError::CommandExecutionError(
                        command,
                        format!("Track {} does not exist", track_idx),
                    ),
                ))?;
                info!("[🛤️ {}] Tempo scale -> {}", track_idx + 1, scale);
                track.set_tempo_scale(scale);
                self.beatmaker.reload_beat_sorter();
            }
            Command::SetChannel(track_idx, channel) => {
                let binding = self.project.tracks();
                let mut trackmap = binding.write().unwrap();
                let track = get_track(&mut trackmap, track_idx).ok_or(SSError::CommandError(
                    crate::error::CommandError::CommandExecutionError(
                        command,
                        format!("Track {} does not exist", track_idx),
                    ),
                ))?;
                info!("[🛤️ {}] Channel -> {}", track_idx + 1, channel + 1);
                track.set_default_channel(channel);
            }
            Command::SetNote(track_idx, note) => {
                let binding = self.project.tracks();
                let mut trackmap = binding.write().unwrap();
                let track = get_track(&mut trackmap, track_idx).ok_or(SSError::CommandError(
                    crate::error::CommandError::CommandExecutionError(
                        command,
                        format!("Track {} does not exist", track_idx),
                    ),
                ))?;
                info!("[🛤️ {}] Note -> {}", track_idx + 1, note);
                track.set_default_note(note);
            }
            Command::SetVelocity(track_idx, velocity) => {
                let binding = self.project.tracks();
                let mut trackmap = binding.write().unwrap();
                let track = get_track(&mut trackmap, track_idx).ok_or(SSError::CommandError(
                    crate::error::CommandError::CommandExecutionError(
                        command,
                        format!("Track {} does not exist", track_idx),
                    ),
                ))?;
                info!("[🛤️ {}] Velocity -> {}", track_idx + 1, velocity);
                track.set_default_velocity(velocity);
            }
            _ => {
                error!("Unsupported command: {}", command);
            }
        }

        Ok(())
    }
}
