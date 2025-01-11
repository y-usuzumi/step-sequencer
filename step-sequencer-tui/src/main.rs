mod tui;
mod ui;
use std::sync::OnceLock;

use crossbeam::channel::{unbounded, Sender};
use log::info;
use step_sequencer::{
    audio::Command,
    beatmaker::pattern::{
        ExampleDrumTracks, EXAMPLE_DRUMTRACKS_BITWIG, EXAMPLE_DRUMTRACKS_GARAGEBAND,
    },
    error::{CommandError, SSError},
    launcher::SSLauncher,
    midi::{note::Note, Channel, Velocity},
    project::F,
    timeline::TimelineState,
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
    let (tui_log_sender, tui_log_receiver) = unbounded();
    let logger = create_tui_logger(tui_log_sender);
    log::set_logger(logger)
        .map(|()| log::set_max_level(log::LevelFilter::Info))
        .unwrap();
    let mut ss_launcher = SSLauncher::new();
    let project = ss_launcher.project();
    let beat_receiver = ss_launcher.subscribe_beatmaker_signals();
    let example_drumtracks = if cfg!(target_os = "linux") {
        &EXAMPLE_DRUMTRACKS_BITWIG
    } else {
        &EXAMPLE_DRUMTRACKS_GARAGEBAND
    };
    for track in example_drumtracks.all_tracks() {
        project.add_track(track);
    }
    ss_launcher.start()?;
    let mut tui = Tui::new(project);
    tui.run_tui(beat_receiver, tui_log_receiver, |s: &str| {
        let command = str_to_command(s);
        let timeline = ss_launcher.timeline();
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
            Ok(Command::Quit) => {
                info!("Quit");
                ss_launcher.stop()
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
            "quit" => Ok(Command::Quit),
            "add_track" => Ok(Command::AddTrack),
            "debug" => Ok(Command::Debug),
            "R" => {
                // "(R)ename track"
                if args.len() >= 2 {
                    let track = args[0].parse::<usize>()? - 1;
                    Ok(Command::RenameTrack(track, args[1].to_string()))
                } else {
                    Err(SSError::CommandError(CommandError::ArgumentError(
                        command.to_string(),
                        args.join(" "),
                    )))
                }
            }
            "t" => {
                // "(T)empo"
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
                // "toggle (B)eat"
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
                // (R)esize
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
            "tc" => {
                // set (T)rack (C)hannel
                if args.len() >= 2 {
                    let track = args[0].parse::<usize>()? - 1;
                    let channel = args[1].parse::<Channel>()?;
                    Ok(Command::SetChannel(track, channel))
                } else {
                    Err(SSError::CommandError(CommandError::ArgumentError(
                        command.to_string(),
                        args.join(" "),
                    )))
                }
            }
            "tn" => {
                // set (T)rack (N)ote
                if args.len() >= 2 {
                    let track = args[0].parse::<usize>()? - 1;
                    let note = args[1].parse::<Note>()?;
                    Ok(Command::SetNote(track, note))
                } else {
                    Err(SSError::CommandError(CommandError::ArgumentError(
                        command.to_string(),
                        args.join(" "),
                    )))
                }
            }
            "ts" => {
                if args.len() >= 2 {
                    let track = args[0].parse::<usize>()? - 1;
                    let numer = args[1].parse::<u64>()?;
                    let denom = if args.len() >= 3 {
                        args[2].parse::<u64>()?
                    } else {
                        1
                    };
                    Ok(Command::TempoScale(track, F::new(numer, denom)))
                } else {
                    Err(SSError::CommandError(CommandError::ArgumentError(
                        command.to_string(),
                        args.join(" "),
                    )))
                }
            }
            "tv" => {
                // set (T)rack (V)elocity
                if args.len() >= 2 {
                    let track = args[0].parse::<usize>()? - 1;
                    let velocity = args[1].parse::<Velocity>()?;
                    Ok(Command::SetVelocity(track, velocity))
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
