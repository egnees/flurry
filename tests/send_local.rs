struct SendLocalProcess {}

impl flurry::Process for SendLocalProcess {
    fn on_message(&mut self, _from: flurry::ProcessId, _msg: String) {
        unreachable!()
    }

    fn on_local_message(&mut self, msg: &str) {
        flurry::send_local(msg.to_string());
    }
}

#[test]
fn send_local() {
    let mut system = flurry::System::default();
    let proc = system.add_process(SendLocalProcess {});

    system.send_local_message(proc, "msg1");
    system.send_local_message(proc, "msg2");
    system.send_local_message(proc, "msg3");

    let msgs = system.read_local(proc);
    assert_eq!(msgs.len(), 3);
    assert_eq!(msgs[0], "msg1");
    assert_eq!(msgs[1], "msg2");
    assert_eq!(msgs[2], "msg3");

    assert!(system.read_local(proc).is_empty());

    let trace = system.get_trace();
    assert_eq!(trace.len(), 6);

    assert_eq!(trace[0].time, 0.0);
    assert_eq!(trace[1].time, 0.0);

    assert_eq!(trace[2].time, 1.0);
    assert_eq!(trace[3].time, 1.0);

    assert_eq!(trace[4].time, 2.0);
    assert_eq!(trace[5].time, 2.0);
}
