use thiserror::Error;

#[derive(Error, Debug)]
pub enum SSError {
    #[error("Unsupported platform: `{0}`")]
    UnsupportedPlatform(String),
    #[error("Unknown: todo")]
    Unknown,
}
