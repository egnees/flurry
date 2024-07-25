use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    rc::{Rc, Weak},
    sync::Arc,
};

use futures::{task::waker, Future};

use crate::{
    ack::AckHandle,
    event::{Event, EventKind, MessageId},
    join::JoinHandle,
    process::{Process, ProcessId},
    shared::SharedState,
    task::{Task, TaskId},
    waker::Waker,
};

/// Represents state of the system,
/// which handles [`SystemHandle`] shared between wakers [`crate::waker::Waker`]
/// and can be accessed by user indirectly using [`System`].
#[derive(Default)]
pub(crate) struct SystemState {
    pending_tasks: VecDeque<TaskId>,
    next_task_id: TaskId,
    tasks: HashMap<TaskId, Task>,
    /// Represents process which is owner
    /// of the currently executing task
    /// or which's method ([`Process::on_local_message`] or [`Process::on_message`])
    /// is called now.
    current_process: Option<ProcessId>,
    local_messages: HashMap<ProcessId, Vec<String>>,
    trace: Vec<Event>,
    time: f64,
    next_msg_id: MessageId,
    pending_events: Vec<EventKind>,
    waiting_ack: HashMap<MessageId, Weak<RefCell<SharedState<bool>>>>,
}

#[derive(Clone)]
pub(crate) struct SystemHandle(Weak<RefCell<SystemState>>);

thread_local! {
    static SYSTEM_HANDLE: RefCell<Option<SystemHandle>> = RefCell::new(None);
}

impl SystemHandle {
    pub(crate) fn current() -> Self {
        SYSTEM_HANDLE.with(|h| h.borrow().as_ref().expect("no system available").clone())
    }

    fn upgrade(&self) -> Rc<RefCell<SystemState>> {
        self.0.upgrade().expect("system is not available")
    }

    pub(crate) fn add_event_kind(&mut self, event_kind: EventKind) {
        let this = self.upgrade();
        let mut state = this.borrow_mut();
        let time = state.time;
        state.trace.push(Event {
            time,
            kind: event_kind,
        });
    }

    pub(crate) fn inc_time(&mut self) {
        self.upgrade().borrow_mut().time += 1.0;
    }

    pub(crate) fn get_trace(&self) -> Vec<Event> {
        self.upgrade().borrow().trace.clone()
    }

    pub(crate) fn send_local(&mut self, msg: String) {
        let this = self.upgrade();
        let mut state = this.borrow_mut();
        let proc = state.current_process.expect(
            "trying to send local message, 
            but current process is not set",
        );

        state
            .local_messages
            .entry(proc)
            .or_insert(Vec::new())
            .push(msg.clone());

        let time = state.time;
        state.trace.push(Event {
            time,
            kind: EventKind::ProcLocalMessage(proc, msg),
        });
    }

    pub(crate) fn schedule(&self, task_id: TaskId) {
        self.upgrade().borrow_mut().pending_tasks.push_back(task_id);
    }

    pub(crate) fn spawn<F>(&self, future: F) -> JoinHandle<F::Output>
    where
        F: Future + 'static,
    {
        let this = self.upgrade();
        let mut state = this.borrow_mut();
        let cur_proc = state.current_process.expect(
            "trying to spawn async activity, 
            but `current_process` is not set",
        );
        let (handle, task) = Task::from_future(cur_proc, future);
        let id = state.next_task_id;
        state.next_task_id += 1;
        state.tasks.insert(id, task);
        state.pending_tasks.push_back(id);
        handle
    }

    pub(crate) fn send(&mut self, to: ProcessId, msg: String) -> AckHandle {
        let this = self.upgrade();
        let mut state = this.borrow_mut();

        let from = state.current_process.expect(
            "trying to send message, 
            but `current_process` is not set",
        );

        let flag = Rc::new(RefCell::new(SharedState::default()));
        let flag_ref = Rc::downgrade(&flag);

        let msg_id = state.next_msg_id;
        state.next_msg_id += 1;
        let old = state.waiting_ack.insert(msg_id, flag_ref);
        assert!(old.is_none(), "duplicate message id: {msg_id}");

        let time = state.time;
        state.trace.push(Event {
            time,
            kind: EventKind::MessageSent(from, to, msg_id, msg.clone()),
        });

        state
            .pending_events
            .push(EventKind::MessageDelivered(from, to, msg_id, msg));

        AckHandle { flag }
    }

    pub(crate) fn get_pending_events(&self) -> Vec<EventKind> {
        self.upgrade().borrow().pending_events.clone()
    }

    pub(crate) fn get_pending_events_count(&self) -> usize {
        self.upgrade().borrow().pending_events.len()
    }

    pub(crate) fn apply_pending_event(&self, event: usize) -> Option<EventKind> {
        let this = self.upgrade();
        let mut state = this.borrow_mut();

        let event_kind = state.pending_events.remove(event);

        let time = state.time;
        state.trace.push(Event {
            time,
            kind: event_kind.clone(),
        });

        match event_kind {
            EventKind::ProcLocalMessage(_, _)
            | EventKind::UserLocalMessage(_, _)
            | EventKind::MessageSent(_, _, _, _)
            | EventKind::AckSent(_, _, _) => panic!("event can not be pending"),
            EventKind::MessageDelivered(from, to, msg_id, _) => {
                state.trace.push(Event {
                    time,
                    kind: EventKind::AckSent(to, from, msg_id),
                });
                state
                    .pending_events
                    .push(EventKind::AckDelivered(to, from, msg_id));
            }
            EventKind::AckDelivered(_, _, msg_id) => {
                drop(state);
                let waiter_ref = this.borrow_mut().waiting_ack.remove(&msg_id).expect(
                    format!("ack waiter is not registered for message with id {msg_id}").as_str(),
                );
                waiter_ref
                    .upgrade()
                    .map(|waiter| waiter.borrow_mut().put(true));
            }
        }

        return Some(event_kind);
    }
}

#[derive(Default)]
pub struct System {
    state: Rc<RefCell<SystemState>>,
    proc: Vec<Box<dyn Process>>,
}

impl System {
    pub fn add_process<P>(&mut self, process: P) -> ProcessId
    where
        P: Process + 'static,
    {
        let id = self.proc.len();
        self.proc.push(Box::new(process));
        id
    }

    fn handle(&self) -> SystemHandle {
        SystemHandle(Rc::downgrade(&self.state))
    }

    fn install_handle(&self) {
        SYSTEM_HANDLE.with(|h| {
            *h.borrow_mut() = Some(self.handle());
        });
    }

    fn set_current_proc(&self, proc: ProcessId) {
        self.state.borrow_mut().current_process = Some(proc);
    }

    pub fn send_local_message(&mut self, to: ProcessId, msg: &str) {
        self.set_current_proc(to);
        self.install_handle();

        self.handle()
            .add_event_kind(EventKind::UserLocalMessage(to, msg.to_string()));

        self.proc
            .get_mut(to)
            .expect("incorrect process id")
            .on_local_message(msg);

        self.process_pending_tasks();
    }

    fn process_pending_task(&mut self) -> bool {
        let (task_id, mut task) = {
            let mut state = self.state.borrow_mut();
            let Some(task_id) = state.pending_tasks.pop_front() else {
                return false;
            };
            let Some(task) = state.tasks.remove(&task_id) else {
                panic!("missing task: {task_id}");
            };
            (task_id, task)
        };
        self.state.borrow_mut().current_process = Some(task.owner());
        let handle = Rc::downgrade(&self.state);
        let waker = waker(Arc::new(Waker {
            system: SystemHandle(handle),
            task_id,
        }));
        let mut ctx = std::task::Context::from_waker(&waker);
        if task.future().as_mut().poll(&mut ctx).is_pending() {
            self.state.borrow_mut().tasks.insert(task_id, task);
        }
        true
    }

    fn process_pending_tasks(&mut self) -> u64 {
        self.install_handle();

        let mut cnt = 0;
        loop {
            if !self.process_pending_task() {
                break;
            }
            cnt += 1;
        }

        self.handle().inc_time();

        cnt
    }

    pub fn read_local(&mut self, proc: ProcessId) -> Vec<String> {
        if self.proc.len() <= proc {
            panic!(
                "trying to read local message 
                from process with incorrect id: {proc}"
            );
        }
        self.state
            .borrow_mut()
            .local_messages
            .entry(proc)
            .or_insert(Vec::new())
            .drain(..)
            .collect()
    }

    pub fn get_trace(&self) -> Vec<Event> {
        self.handle().get_trace()
    }

    pub fn get_pending_events(&self) -> Vec<EventKind> {
        self.handle().get_pending_events()
    }

    pub fn get_pending_events_count(&self) -> usize {
        self.handle().get_pending_events_count()
    }

    pub fn apply_pending_event(&mut self, event: usize) {
        self.handle()
            .apply_pending_event(event)
            .map(|event| match event {
                EventKind::MessageDelivered(from, to, _, msg) => {
                    self.set_current_proc(to);

                    self.proc
                        .get_mut(to)
                        .expect("invalid process id")
                        .on_message(from, msg);
                }
                _ => {}
            });
        self.process_pending_tasks();
    }
}
