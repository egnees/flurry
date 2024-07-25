use crate::{ack::AckHandle, system::SystemHandle, ProcessId};

pub fn send_local(msg: String) {
    SystemHandle::current().send_local(msg)
}

pub fn send(to: ProcessId, msg: String) -> AckHandle {
    SystemHandle::current().send(to, msg)
}
