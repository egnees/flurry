use std::{collections::HashSet, time::Instant};

use process::BroadcastProcess;

mod process;

use rand::Rng;

fn main() {
    let now = Instant::now();

    let mut rng = rand::thread_rng();

    let proc_cnt = 200;
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
        let start_from = rng.gen::<usize>() % proc_cnt;
        system.send_local_message(start_from, content.as_str());

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

    let processed_tasks = system.get_processed_tasks();
    let processed_events = system.get_processed_events_count();
    let total_processed = processed_tasks + processed_events;
    let elapsed = now.elapsed();

    println!("Processed tasks: {processed_tasks}");
    println!("Processed events: {processed_events}");
    println!("Total processed: {total_processed}");
    println!("Elapsed: {:.2?}", elapsed);
    println!(
        "Processed/s: {:.2}",
        (total_processed as f64) / elapsed.as_secs_f64()
    );
}
