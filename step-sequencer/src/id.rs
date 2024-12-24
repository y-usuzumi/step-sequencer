use uuid::Uuid;

pub type SSId = Uuid;

pub fn new_id() -> SSId {
    Uuid::new_v4()
}