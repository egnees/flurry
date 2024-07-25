/// User must put the value exactly once and
/// take the value no more than once.

#[derive(Default)]
pub(crate) enum SharedState<T> {
    #[default]
    Initial,
    Ready(T),
    Waiting(std::task::Waker),
}

impl<T> SharedState<T> {
    /// If value is already put, then value is returned.
    /// Else, waker will be stored and called after value will be put.
    pub(crate) fn take(&mut self, waker: std::task::Waker) -> Option<T> {
        let old = std::mem::replace(self, SharedState::Waiting(waker));
        match old {
            SharedState::Initial => None,
            SharedState::Ready(value) => Some(value),
            SharedState::Waiting(_) => panic!("taking value twice is contract violation"),
        }
    }

    /// Store provided value.
    /// If there is registered waker, it will be waked.
    pub(crate) fn put(&mut self, value: T) {
        let old = std::mem::replace(self, SharedState::Ready(value));
        match old {
            SharedState::Initial => {}
            SharedState::Ready(_) => panic!("putting value twice is contract violation"),
            SharedState::Waiting(waker) => waker.wake(),
        }
    }
}
