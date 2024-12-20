use crate::{audio::SSClient, SSResult};

pub struct SSCoreAudioClient;

impl SSCoreAudioClient {
    pub fn new() -> Self {
        Self
    }
}

impl SSClient for SSCoreAudioClient {
    fn start(&self) -> SSResult<()> {
        println!("SSCoreAudioClient started");
        Ok(())
    }
}
