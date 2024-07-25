struct SpawnProcess {}

impl flurry::Process for SpawnProcess {
    fn on_message(&mut self, _: flurry::ProcessId, _: &str) {
        unreachable!()
    }

    fn on_local_message(&mut self, _: &str) {
        flurry::spawn(async move {
            flurry::send_local(format!("spawn1"));
            let res2 = flurry::spawn(async move {
                flurry::send_local(format!("spawn2"));
                2
            });
            let res3 = flurry::spawn(async move {
                flurry::send_local(format!("spawn3"));
                res2.await + 3
            });
            let total_result = res3.await + 1; // must be 2+3+1
            flurry::send_local(format!("total_result: {total_result}"));
        });
    }
}

#[test]
fn spawn() {
    let mut sys = flurry::System::default();
    let proc = sys.add_process(SpawnProcess {});
    sys.send_local_message(proc, "some msg");
    let msgs = sys.read_local(proc);
    assert_eq!(msgs.len(), 4);
    assert_eq!(msgs[0], "spawn1");
    assert_eq!(msgs[1], "spawn2");
    assert_eq!(msgs[2], "spawn3");
    assert_eq!(msgs[3], "total_result: 6");
}
