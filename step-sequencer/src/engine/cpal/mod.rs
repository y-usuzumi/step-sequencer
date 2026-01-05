use std::sync::Arc;

use crate::{SSResult, beatmaker::BeatMakerSubscription, engine::SSClient};

pub struct SSCpalClient {
    beatmaker_subscription: Arc<BeatMakerSubscription>,
}

impl SSCpalClient {
    pub fn new(beatmaker_subscription: BeatMakerSubscription) -> Self {
        Self {
            beatmaker_subscription: Arc::new(beatmaker_subscription),
        }
    }
}

impl SSClient for SSCpalClient {
    fn start(&mut self) -> SSResult<()> {
        SSResult::Ok(())
    }
    fn stop(&mut self) -> SSResult<()> {
        SSResult::Ok(())
    }
}
