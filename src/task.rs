use std::{cell::RefCell, pin::Pin, rc::Rc};

use futures::Future;

use crate::{join::JoinHandle, process::ProcessId, shared::SharedState};

type BoxedFuture = Pin<Box<dyn Future<Output = ()>>>;

/// Represents asynchronous task created by certain process.
pub(crate) type TaskId = usize;

pub(crate) struct Task(ProcessId, BoxedFuture);

impl Task {
    pub(crate) fn from_future<F>(proc: ProcessId, future: F) -> (JoinHandle<F::Output>, Task)
    where
        F: Future + 'static,
    {
        let result = Rc::new(RefCell::new(SharedState::default()));
        let result_ref = Rc::downgrade(&result);
        let join_handle = JoinHandle { result };

        let future = async move {
            let value = future.await;
            result_ref
                .upgrade()
                .map(|result| result.borrow_mut().put(value));
        };

        (join_handle, Task(proc, Box::pin(future)))
    }

    pub(crate) fn owner(&self) -> ProcessId {
        self.0
    }

    pub(crate) fn future(&mut self) -> &mut Pin<Box<dyn Future<Output = ()>>> {
        &mut self.1
    }
}
