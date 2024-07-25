use crate::system::SystemHandle;

pub fn send_local(msg: String) {
    SystemHandle::current().send_local(msg)
}
