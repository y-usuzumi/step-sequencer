use std::rc::Rc;

use derive_builder::Builder;
use log::{error, info};

use crate::{
    audio::{create_ss_client, Command, SSClient},
    beatmaker::BeatMaker,
    error::SSError,
    project::Project,
    timeline::{Timeline, TimelineState},
    SSResult,
};

#[derive(Builder)]
#[builder(pattern = "owned")]
pub struct SSLauncher {
    timeline: Rc<Timeline>,
    beatmaker: Rc<BeatMaker>,
    project: Rc<Project>,
}

impl SSLauncher {
    pub fn start(&self) -> SSResult<()> {
        let client = create_ss_client(self.beatmaker.clone())?;
        client.start()?;
        self.beatmaker
            .start(&self.project, self.timeline.subscribe());
        self.timeline.start();
        Ok(())
    }

    pub fn send_command(&self, command: Command) -> SSResult<()> {
        // TODO: Copy-paste from CoreAudio side. Improve the design
        match command {
            Command::ChangeTempo(tempo) => {
                let project_settings = self.project.project_settings();
                project_settings.write().unwrap().tempo = tempo;
            }
            Command::ToggleBeat(track, beat) => {
                let binding = self.project.tracks();
                let mut trackmap = binding.write().unwrap();
                let mut tracks = trackmap.values_mut();
                for _ in 0..track {
                    tracks.next();
                }
                let track = tracks.next().ok_or(SSError::CommandError(
                    crate::error::CommandError::CommandExecutionError(
                        command,
                        format!("Track {} does not exist", track),
                    ),
                ))?;
                track.toggle_beat(beat);
            }
            Command::Resize(track, size) => {
                let binding = self.project.tracks();
                let mut trackmap = binding.write().unwrap();
                let mut tracks = trackmap.values_mut();
                for _ in 0..track {
                    tracks.next();
                }
                let track = tracks.next().ok_or(SSError::CommandError(
                    crate::error::CommandError::CommandExecutionError(
                        command,
                        format!("Track {} does not exist", track),
                    ),
                ))?;
                track.resize(size);
            }
            Command::PlayOrPause => match self.timeline.state() {
                TimelineState::Stopped => {
                    info!("Start");
                    let _ = self.timeline.start();
                }
                TimelineState::Started => {
                    info!("Pause");
                    let _ = self.timeline.pause();
                }
            },
            _ => {
                error!("Unsupported command: {}", command);
            }
        }

        Ok(())
    }
}
