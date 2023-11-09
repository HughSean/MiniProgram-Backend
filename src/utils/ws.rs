use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum Table {
    Order,
    Court,
}
#[derive(Clone, Debug)]
pub enum Event {
    Update,
    Add,
    Del,
}

#[derive(Clone, Debug)]
pub struct Msg {
    pub event: Event,
    pub table: Table,
    pub model_id: Uuid,
}
