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

struct ReorderSpawnProcess {}

impl flurry::Process for ReorderSpawnProcess {
    fn on_message(&mut self, _: flurry::ProcessId, _: &str) {
        unreachable!()
    }

    fn on_local_message(&mut self, _: &str) {
        flurry::spawn(async move {
            let handle = flurry::spawn(async move {
                flurry::spawn(async move {
                    flurry::send_local("send2".to_string());
                })
                .await;
            });
            flurry::send_local("send1".to_string());
            handle.await;
            flurry::send_local("send3".to_string());
        });
    }
}

#[test]
fn reordered_spawns() {
    let mut sys = flurry::System::default();
    let proc = sys.add_process(ReorderSpawnProcess {});
    sys.send_local_message(proc, "some msg");
    let msgs = sys.read_local(proc);
    assert_eq!(msgs.len(), 3);
    assert_eq!(msgs[0], "send1");
    assert_eq!(msgs[1], "send2");
    assert_eq!(msgs[2], "send3");
}
