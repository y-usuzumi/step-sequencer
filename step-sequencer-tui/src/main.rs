mod command;
mod tui;
mod ui;
use std::sync::OnceLock;

use command::str_to_command;
use crossbeam::channel::{unbounded, Sender};
use log::info;
use step_sequencer::{
    beatmaker::pattern::{
        ExampleDrumTracks, EXAMPLE_DRUMTRACKS_BITWIG, EXAMPLE_DRUMTRACKS_GARAGEBAND,
    },
    error::{CommandError, SSError},
    launcher::SSLauncher,
    timeline::TimelineState,
    SSResult,
};
use tui::{Tui, TuiLogger};

fn create_tui_logger(sender: Sender<String>) -> &'static TuiLogger {
    static TUI_LOGGER: OnceLock<TuiLogger> = OnceLock::new();
    TUI_LOGGER.get_or_init(|| TuiLogger::new(sender))
}

fn main() -> SSResult<()> {
    run_main()
}

#[cfg(not(feature = "precise_timing"))]
fn run_main() -> SSResult<()> {
    // Need to use a more versatile logger to be able to write to logger in tui.
    // Now disabling env_logger temporarily and write only to my tui custom logger.
    // env_logger::init();

    use step_sequencer::launcher::SSLauncherImpl;
    let (tui_log_sender, tui_log_receiver) = unbounded();
    let logger = create_tui_logger(tui_log_sender);
    log::set_logger(logger)
        .map(|()| log::set_max_level(log::LevelFilter::Info))
        .unwrap();
    let mut ss_launcher = SSLauncherImpl::new();
    let project = ss_launcher.project();
    let beat_receiver = ss_launcher.subscribe_to_beats();
    let example_drumtracks = if cfg!(target_os = "linux") {
        &EXAMPLE_DRUMTRACKS_BITWIG
    } else {
        &EXAMPLE_DRUMTRACKS_GARAGEBAND
    };
    for track in example_drumtracks.all_tracks() {
        project.add_track(track);
    }
    ss_launcher.start()?;
    let mut tui = Tui::new(&mut ss_launcher);
    tui.run_tui(beat_receiver, tui_log_receiver, |ss_launcher, s: &str| {
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

#[cfg(feature = "precise_timing")]
fn run_main() -> SSResult<()> {
    // Need to use a more versatile logger to be able to write to logger in tui.
    // Now disabling env_logger temporarily and write only to my tui custom logger.
    // env_logger::init();

    use step_sequencer::{command::Command, launcher::SSLauncherImpl2};
    let (tui_log_sender, tui_log_receiver) = unbounded();
    let logger = create_tui_logger(tui_log_sender);
    log::set_logger(logger)
        .map(|()| log::set_max_level(log::LevelFilter::Info))
        .unwrap();
    let mut ss_launcher = SSLauncherImpl2::new();
    let project = ss_launcher.project();
    let beat_receiver = ss_launcher.subscribe_to_beats();
    let example_drumtracks = if cfg!(target_os = "linux") {
        &EXAMPLE_DRUMTRACKS_BITWIG
    } else {
        &EXAMPLE_DRUMTRACKS_GARAGEBAND
    };
    for track in example_drumtracks.all_tracks() {
        project.add_track(track);
    }
    ss_launcher.start()?;
    let mut tui = Tui::new(&mut ss_launcher);
    tui.run_tui(beat_receiver, tui_log_receiver, |ss_launcher, s: &str| {
        let command = str_to_command(s);
        match command {
            Err(SSError::CommandError(CommandError::EmptyCommand)) => Ok(()),
            Err(e) => Err(e),
            Ok(Command::PlayOrPause) => {
                info!("TODO: implement PlayOrPause");
                Ok(())
            }
            Ok(Command::Stop) => {
                info!("Stop");
                ss_launcher.stop();
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
