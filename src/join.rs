use std::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use crate::shared::SharedState;

pub struct JoinHandle<T> {
    /// Result item lifetime is bounded by the `JoinHandle` lifetime.
    pub(crate) result: Rc<RefCell<SharedState<T>>>,
}

impl<T> Future for JoinHandle<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(value) = self.result.borrow_mut().take(cx.waker().clone()) {
            Poll::Ready(value)
        } else {
            // here waker is registered in the shared state
            Poll::Pending
        }
    }
}
