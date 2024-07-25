use futures::Future;

use crate::{join::JoinHandle, system::SystemHandle};

pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + 'static,
{
    SystemHandle::current().spawn(future)
}
