use std::{
    cell::RefCell,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use futures::Future;

use crate::shared::SharedState;

pub struct AckHandle {
    pub(crate) flag: Rc<RefCell<SharedState<bool>>>,
}

impl Future for AckHandle {
    type Output = bool;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(value) = self.flag.borrow_mut().take(cx.waker().clone()) {
            Poll::Ready(value)
        } else {
            Poll::Pending
        }
    }
}
