mod tui;
mod widgets;
use std::{
    rc::Rc,
    sync::{
        mpsc::{self, Sender},
        OnceLock,
    },
};

use log::info;
use step_sequencer::{
    audio::{create_ss_client, Command, SSClient},
    beatmaker::{
        pattern::{ExampleDrumTracks, EXAMPLE_DRUMTRACKS_BITWIG, EXAMPLE_DRUMTRACKS_GARAGEBAND},
        BeatMaker,
    },
    error::{CommandError, SSError},
    launcher::{SSLauncher, SSLauncherBuilder},
    project::Project,
    timeline::{Timeline, TimelineState},
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
    let (tui_log_sender, tui_log_receiver) = mpsc::channel();
    let logger = create_tui_logger(tui_log_sender);
    log::set_logger(logger)
        .map(|()| log::set_max_level(log::LevelFilter::Info))
        .unwrap();
    let timeline = Rc::new(Timeline::default());
    let beatmaker = Rc::new(BeatMaker::new());
    let project = Rc::new(Project::new());
    let example_drumtracks = if cfg!(target_os = "linux") {
        &EXAMPLE_DRUMTRACKS_BITWIG
    } else {
        &EXAMPLE_DRUMTRACKS_GARAGEBAND
    };
    for track in example_drumtracks.all_tracks() {
        project.add_track(track);
    }
    let beat_receiver = beatmaker.subscribe_beats();
    let ss_launcher = SSLauncherBuilder::default()
        .timeline(Rc::clone(&timeline))
        .beatmaker(Rc::clone(&beatmaker))
        .project(Rc::clone(&project))
        .build()
        .unwrap();
    ss_launcher.start()?;
    let mut tui = Tui::new(project);
    tui.run_tui(beat_receiver, tui_log_receiver, |s: &str| {
        let command = str_to_command(s);
        match command {
            Err(SSError::CommandError(CommandError::EmptyCommand)) => Ok(()),
            Err(e) => Err(e),
            Ok(Command::PlayOrPause) => match timeline.state() {
                TimelineState::Stopped => {
                    info!("Start");
                    timeline.start();
                    Ok(())
                }
                TimelineState::Started => {
                    info!("Pause");
                    timeline.pause();
                    Ok(())
                }
            },
            Ok(Command::Stop) => {
                info!("Stop");
                timeline.stop();
                Ok(())
            }
            Ok(command) => ss_launcher.send_command(command),
        }
    })
}

fn str_to_command(s: &str) -> SSResult<Command> {
    let mut chunks = s.split_whitespace();
    if let Some(command) = chunks.next() {
        let args: Vec<&str> = chunks.collect();
        match command {
            "play" => Ok(Command::PlayOrPause),
            "stop" => Ok(Command::Stop),
            "t" => {
                if args.len() >= 1 {
                    let tempo = args[0].parse::<u16>()?;
                    Ok(Command::ChangeTempo(tempo))
                } else {
                    Err(SSError::CommandError(CommandError::ArgumentError(
                        command.to_string(),
                        args.join(" "),
                    )))
                }
            }
            "b" => {
                if args.len() >= 2 {
                    let track = args[0].parse::<usize>()? - 1;
                    let beat = args[1].parse::<usize>()? - 1;
                    Ok(Command::ToggleBeat(track, beat))
                } else {
                    Err(SSError::CommandError(CommandError::ArgumentError(
                        command.to_string(),
                        args.join(" "),
                    )))
                }
            }
            "r" => {
                if args.len() >= 2 {
                    let track = args[0].parse::<usize>()? - 1;
                    let size = args[1].parse::<usize>()?;
                    Ok(Command::Resize(track, size))
                } else {
                    Err(SSError::CommandError(CommandError::ArgumentError(
                        command.to_string(),
                        args.join(" "),
                    )))
                }
            }
            _ => Err(SSError::CommandError(CommandError::InvalidCommand(
                command.to_string(),
            ))),
        }
    } else {
        Err(SSError::CommandError(CommandError::EmptyCommand))
    }
}
