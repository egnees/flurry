use std::{collections::HashSet, time::Instant};

use process::BroadcastProcess;

mod process;

use rand::Rng;

fn main() {
    let now = Instant::now();

    let mut rng = rand::thread_rng();

    let proc_cnt = 100;
    let messages = 10;

    let mut system = flurry::System::default();

    let all = (0..proc_cnt).collect::<Vec<_>>();
    for proc in 0..proc_cnt {
        let mut except_this = all.clone();
        except_this.remove(proc);
        let proc = BroadcastProcess {
            others: except_this,
            delivered: HashSet::new(),
        };
        system.add_process(proc);
    }

    for msg in 1..=messages {
        let content = format!("message number {msg}");
        system.send_local_message(rng.gen::<usize>() % proc_cnt, content.as_str());

        loop {
            let pending_events_cnt = system.get_pending_events_count();
            if pending_events_cnt == 0 {
                break;
            }
            let event = rng.gen::<usize>() % pending_events_cnt;
            system.apply_pending_event(event);
        }
    }

    for proc in 0..proc_cnt {
        let delivered = system.read_local(proc);
        assert_eq!(delivered.len(), messages);
        for i in 0..delivered.len() {
            assert_eq!(delivered[i], format!("message number {}", i + 1));
        }
    }

    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}
