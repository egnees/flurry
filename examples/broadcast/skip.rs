pub fn can_skip(sys: &mut flurry::System, proc_cnt: usize) -> bool {
    for proc in 0..proc_cnt {
        let msgs = sys.read_local(proc);
        if msgs.len() == 0 {
            return false;
        }
    }
    true
}
