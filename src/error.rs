use std::io;
use thiserror::Error;

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
    #[error("Unsupported platform: `{0}`")]
    UnsupportedPlatform(String),
    #[error("Unknown: todo")]
    Unknown,
}
