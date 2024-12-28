use uuid::Uuid;

pub type SSId = Uuid;

pub fn new_id() -> SSId {
    Uuid::new_v4()
}

pub type AutoIncrementId = u64;

pub struct AutoIncrementIdGen {
    current: u64,
}

impl AutoIncrementIdGen {
    pub fn new() -> Self {
        Self { current: 0 }
    }

    pub fn next(&mut self) -> AutoIncrementId {
        let next = self.current;
        self.current += 1;
        return next;
    }
}

impl Default for AutoIncrementIdGen {
    fn default() -> Self {
        AutoIncrementIdGen::new()
    }
}
