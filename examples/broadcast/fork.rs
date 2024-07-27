mod check;
mod process;
mod skip;

use std::time::Instant;

use nix::libc::{exit, fork, wait};
use process::BroadcastProcess;

fn search(proc_cnt: usize) -> bool {
    let mut sys = flurry::System::default();
    let all: Vec<_> = (0..proc_cnt).collect();
    for proc in 0..proc_cnt {
        let mut except = all.clone();
        except.remove(proc);
        sys.add_process(BroadcastProcess {
            others: except,
            delivered: std::collections::HashSet::default(),
        });
    }
    sys.send_local_message(0, "message");
    let mut was_child = false;
    loop {
        if skip::can_skip(&mut sys, proc_cnt) {
            unsafe { exit(0) };
        }
        let pending_events_cnt = sys.get_pending_events_count();
        if pending_events_cnt == 0 {
            let failed = if check::check(&mut sys, proc_cnt) {
                0
            } else {
                1
            };
            unsafe { exit(failed) };
        }
        let mut in_child = false;
        // println!("pending_events_cnt = {:?}", pending_events_cnt);
        for event in 0..pending_events_cnt {
            let pid = unsafe { fork() };
            if pid != 0 {
            } else {
                // in child
                sys.apply_pending_event(event);
                in_child = true;
                was_child = true;
                break;
            }
        }
        if !in_child {
            let mut status = 0;
            unsafe { wait(&mut status) };
            if was_child {
                unsafe { exit(status) };
            } else {
                if status == 1 {
                    return false;
                } else {
                    return true;
                }
            }
        }
    }
}

fn main() {
    let now = Instant::now();
    let search_result = search(3);
    println!("Search result: {search_result}");
    let elapsed = now.elapsed();
    println!("Elapsed: {:?}", elapsed);
}
