mod check;
mod process;
mod state;

use std::{collections::VecDeque, time::Instant};

use process::BroadcastProcess;
use state::State;

fn make_system(proc_cnt: usize) -> flurry::System {
    let mut sys = flurry::System::default();
    let add = (0..proc_cnt).collect::<Vec<flurry::ProcessId>>();
    for proc in 0..proc_cnt {
        let mut except = add.clone();
        except.remove(proc);
        sys.add_process(BroadcastProcess {
            others: except,
            delivered: std::collections::HashSet::default(),
        });
    }
    sys.send_local_message(0, "message");
    sys
}

fn make_system_with_state(proc_cnt: usize, state: &State) -> flurry::System {
    let mut sys = make_system(proc_cnt);

    for event in state.to_apply.iter() {
        sys.apply_pending_event(*event);
    }

    sys
}

fn search(proc_cnt: usize) -> (bool, usize) {
    let mut failed = false;

    let mut states = VecDeque::<State>::new();
    let mut processed_states = 0;
    states.push_back(State {
        to_apply: Vec::default(),
    });

    while !states.is_empty() {
        let state = states.pop_front().unwrap();

        processed_states += 1;

        let mut sys = make_system_with_state(proc_cnt, &state);
        if sys.get_pending_events_count() == 0 {
            let check_result = check::check(&mut sys, proc_cnt);
            if !check_result {
                failed = true;
                break;
            }
            continue;
        }

        let pending_events_cnt = sys.get_pending_events_count();
        for i in 0..pending_events_cnt {
            let mut new_state = state.clone();
            new_state.to_apply.push(i);
            states.push_back(new_state);
        }
    }

    (failed, processed_states)
}

fn main() {
    let now = Instant::now();
    let (failed, processed_states) = search(3);
    if failed {
        println!("Failed!");
        return;
    }
    let elapsed = now.elapsed();
    println!("Processed {processed_states} states");
    println!("Elapsed time: {:?}", elapsed);
    println!(
        "Processed/s: {}",
        (processed_states as f64) / elapsed.as_secs_f64()
    );
}
