pub fn check(sys: &mut flurry::System, proc_cnt: usize) -> bool {
    assert_eq!(sys.get_pending_events_count(), 0);
    for proc in 0..proc_cnt {
        let msgs = sys.read_local(proc);
        if msgs.len() != 1 {
            return false;
        }
    }
    true
}
