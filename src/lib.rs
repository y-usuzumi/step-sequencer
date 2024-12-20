use error::SSError;

pub mod audio;
pub mod error;
pub mod midi;

pub type SSResult<T> = std::result::Result<T, SSError>;
