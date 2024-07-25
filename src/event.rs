#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub enum EventKind {
    LocalMessage(String),
    Message(u64, String),
    Ack(u64),
}

#[derive(Debug, Clone)]
pub struct Event {
    pub time: f64,
    pub kind: EventKind,
}
