use std::io;

use log::{error, info};
use step_sequencer::{audio::{create_ss_client, Command}, beatmaker::{pattern::{ExampleDrumTracks, EXAMPLE_DRUMTRACKS_BITWIG, EXAMPLE_DRUMTRACKS_GARAGEBAND}, BeatMaker}, project::Project, SSResult};

fn main() -> SSResult<()> {
    env_logger::init();
    let beatmaker = BeatMaker::default();
    let project = Project::new();
    let example_drumtracks = if cfg!(target_os="linux") {
        &EXAMPLE_DRUMTRACKS_BITWIG
    } else {
        &EXAMPLE_DRUMTRACKS_GARAGEBAND
    };
    for track in example_drumtracks.all_tracks() {
        project.add_track(track);
    }
    let mut ss_client = create_ss_client(beatmaker, project)?;
    ss_client.start()?;
    let mut user_input = String::new();
    loop {
        // 4. Wait for user input to quit
        println!("Enter a command (Q/q to quit)...");
        io::stdin().read_line(&mut user_input).ok();
        match user_input.trim_end() {
            "q" | "Q" => {
                ss_client.stop()?;
                break;
            }
            input => {
                if let Ok(tempo) = input.parse::<u16>() {
                    ss_client.send_command(Command::ChangeTempo(tempo))?;
                }
            }
        }
        user_input.clear();
    }
    Ok(())
}
