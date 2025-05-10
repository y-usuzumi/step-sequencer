use step_sequencer::{
    command::Command,
    error::{CommandError, SSError},
    midi::{note::Note, Channel, Velocity},
    project::F,
    SSResult,
};

pub fn str_to_command(s: &str) -> SSResult<Command> {
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
