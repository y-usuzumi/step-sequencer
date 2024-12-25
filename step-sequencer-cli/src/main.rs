mod tui;
mod widgets;
use std::sync::{
    mpsc::{self, Sender},
    OnceLock,
};

use log::info;
use step_sequencer::{
    audio::{create_ss_client, Command},
    beatmaker::{
        pattern::{ExampleDrumTracks, EXAMPLE_DRUMTRACKS_BITWIG, EXAMPLE_DRUMTRACKS_GARAGEBAND},
        BeatMaker,
    },
    project::Project,
    SSResult,
};
use tui::{Tui, TuiLogger};

fn create_tui_logger(sender: Sender<String>) -> &'static TuiLogger {
    static TUI_LOGGER: OnceLock<TuiLogger> = OnceLock::new();
    TUI_LOGGER.get_or_init(|| TuiLogger::new(sender))
}

fn main() -> SSResult<()> {
    // Need to use a more versatile logger to be able to write to logger in tui.
    // Now disabling env_logger temporarily and write only to my tui custom logger.
    // env_logger::init();
    let (tx, rx) = mpsc::channel();
    let logger = create_tui_logger(tx);
    log::set_logger(logger)
        .map(|()| log::set_max_level(log::LevelFilter::Info))
        .unwrap();
    let beatmaker = BeatMaker::default();
    let project = Project::new();
    let example_drumtracks = if cfg!(target_os = "linux") {
        &EXAMPLE_DRUMTRACKS_BITWIG
    } else {
        &EXAMPLE_DRUMTRACKS_GARAGEBAND
    };
    for track in example_drumtracks.all_tracks() {
        project.add_track(track);
    }
    let mut ss_client = create_ss_client(beatmaker, &project)?;
    ss_client.start()?;
    let mut tui = Tui::new(&project);
    tui.run_tui(rx, |s: &str| {
        let command = str_to_command(s)?;
        info!("Running command: {:?}", command);
        ss_client.send_command(command)
    })
}

fn str_to_command(s: &str) -> SSResult<Command> {
    if let Ok(tempo) = s.parse::<u16>() {
        Ok(Command::ChangeTempo(tempo))
    } else {
        Err(step_sequencer::error::SSError::InvalidCommand(
            s.to_string(),
        ))
    }
}
