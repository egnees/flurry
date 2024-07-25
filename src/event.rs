use crate::ProcessId;

pub type MessageId = usize;

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub enum EventKind {
    ProcLocalMessage(ProcessId, String),
    UserLocalMessage(ProcessId, String),
    MessageSent(ProcessId, ProcessId, MessageId, String),
    MessageDelivered(ProcessId, ProcessId, MessageId, String),
    AckSent(ProcessId, ProcessId, MessageId),
    AckDelivered(ProcessId, ProcessId, MessageId),
}

#[derive(Debug, Clone)]
pub struct Event {
    pub time: f64,
    pub kind: EventKind,
}
