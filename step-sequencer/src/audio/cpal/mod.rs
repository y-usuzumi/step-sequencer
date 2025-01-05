use std::rc::Rc;

use crate::beatmaker::BeatMaker;

pub struct SSCpalClient {
    beatmaker: Rc<BeatMaker>,
}

impl SSCpalClient {
    pub fn new(beatmaker: Rc<BeatMaker>) -> Self {
        Self { beatmaker }
    }
}
