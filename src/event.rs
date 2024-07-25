use crate::ProcessId;

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub enum EventKind {
    ProcLocalMessage(ProcessId, String),
    UserLocalMessage(ProcessId, String),
    Message(ProcessId, ProcessId, u64, String),
    Ack(ProcessId, ProcessId, u64),
}

#[derive(Debug, Clone)]
pub struct Event {
    pub time: f64,
    pub kind: EventKind,
}
