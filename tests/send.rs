use std::{cell::RefCell, rc::Rc};

use flurry::EventKind;

struct SendProcess {
    sent_cnt: Rc<RefCell<usize>>,
    recv_cnt: usize,
    pair: flurry::ProcessId,
}

impl SendProcess {
    fn send(&self, to: flurry::ProcessId, msg: String) {
        let sent_cnt = self.sent_cnt.clone();
        flurry::spawn(async move {
            flurry::send(to, msg).await;
            *sent_cnt.borrow_mut() += 1;
            flurry::send_local(format!("sent: {}", *sent_cnt.borrow()));
        });
    }
}

impl flurry::Process for SendProcess {
    fn on_message(&mut self, from: flurry::ProcessId, msg: String) {
        self.recv_cnt += 1;
        flurry::send_local(format!("received: {}", self.recv_cnt));
        self.send(from, msg);
    }

    fn on_local_message(&mut self, msg: &str) {
        self.send(self.pair, msg.to_string());
    }
}

#[test]
fn send() {
    let mut system = flurry::System::default();

    let proc1 = system.add_process(SendProcess {
        sent_cnt: Rc::new(RefCell::new(0)),
        recv_cnt: 0,
        pair: 1,
    });
    assert_eq!(proc1, 0);

    let proc2 = system.add_process(SendProcess {
        sent_cnt: Rc::new(RefCell::new(0)),
        recv_cnt: 0,
        pair: 0,
    });
    assert_eq!(proc2, 1);

    system.send_local_message(proc1, "some message");
    assert_eq!(system.get_pending_events_count(), 1);
    let trace = system.get_trace();
    assert_eq!(trace.len(), 2);
    assert_eq!(
        trace[0].kind,
        EventKind::UserLocalMessage(proc1, "some message".to_string())
    );
    assert_eq!(
        trace[1].kind,
        EventKind::MessageSent(proc1, proc2, 0, "some message".to_string())
    );

    system.apply_pending_event(0);
    let trace = system.get_trace();
    assert_eq!(trace.len(), 6);
    assert_eq!(
        trace[2].kind,
        EventKind::MessageDelivered(proc1, proc2, 0, "some message".to_string())
    );
    assert_eq!(trace[3].kind, EventKind::AckSent(proc2, proc1, 0));
    assert_eq!(
        trace[4].kind,
        EventKind::ProcLocalMessage(proc2, "received: 1".to_string())
    );
    assert_eq!(
        trace[5].kind,
        EventKind::MessageSent(proc2, proc1, 1, "some message".to_string())
    );

    let proc2_local = system.read_local(proc2);
    assert_eq!(proc2_local.len(), 1);
    assert_eq!(proc2_local[0], "received: 1");

    assert_eq!(system.get_pending_events_count(), 2);
    system.apply_pending_event(0);
    let proc1_local = system.read_local(proc1);
    assert_eq!(proc1_local.len(), 1);
    assert_eq!(proc1_local[0], "sent: 1");

    for i in 1..11 {
        assert_eq!(system.get_pending_events_count(), 1);
        system.apply_pending_event(0); // apply `MessageDelivered`
        let receiver_proc_local = system.read_local((i + 1) % 2);
        assert_eq!(receiver_proc_local.len(), 1);
        assert_eq!(receiver_proc_local[0], format!("received: {}", 1 + i / 2));
        system.apply_pending_event(0); // apply `Ack`
        let sender_proc_local = system.read_local(i % 2);
        assert_eq!(sender_proc_local.len(), 1);
        assert_eq!(sender_proc_local[0], format!("sent: {}", 1 + i / 2));
    }
}
