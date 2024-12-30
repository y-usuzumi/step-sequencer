use std::{io, num::ParseIntError};
use thiserror::Error;

use crate::audio::Command;

pub type SSResult<T> = std::result::Result<T, SSError>;

#[derive(Error, Debug)]
pub enum SSError {
    #[cfg(target_os = "macos")]
    #[error("CoreAudio error: `{0}`")]
    CoreAudioError(#[from] coreaudio::Error),
    #[cfg(target_os = "linux")]
    #[error("JACK error: `{0}`")]
    JackError(#[from] jack::Error),
    #[error("IO error: `{0}`")]
    IOError(#[from] io::Error),
    #[error("Command error: `{0}`")]
    CommandError(#[from] CommandError),
    #[error("Parse int error: `{0}`")]
    ParseIntError(#[from] ParseIntError),
    #[error("Unsupported platform: `{0}`")]
    UnsupportedPlatform(String),
    #[error("Unknown: `{0}`")]
    Unknown(String),
}

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("Empty command")]
    EmptyCommand,
    #[error("Invalid command: `{0}`")]
    InvalidCommand(String),
    #[error("Argument error for command `{0}`: `{1}`")]
    ArgumentError(String, String),
    #[error("Command execution error: `{0}`")]
    CommandExecutionError(Command, String),
}