use thiserror::Error;

#[derive(Error, Debug)]
pub enum SSError {
    #[cfg(target_os = "macos")]
    #[error("CoreAudio error: `{0}`")]
    CoreAudioError(#[from] coreaudio::Error),
    #[error("Unsupported platform: `{0}`")]
    UnsupportedPlatform(String),
    #[error("Unknown: todo")]
    Unknown,
}
