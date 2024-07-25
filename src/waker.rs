use std::sync::Arc;

use futures::task::ArcWake;

use crate::{system::SystemHandle, task::TaskId};

pub(crate) struct Waker {
    pub(crate) system: SystemHandle,
    pub(crate) task_id: TaskId,
}

unsafe impl Sync for Waker {}
unsafe impl Send for Waker {}

impl ArcWake for Waker {
    fn wake(self: Arc<Self>) {
        self.system.schedule(self.task_id);
    }

    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.system.schedule(arc_self.task_id);
    }
}
