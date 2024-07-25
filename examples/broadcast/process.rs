use std::collections::HashSet;

pub struct BroadcastProcess {
    pub others: Vec<flurry::ProcessId>,
    pub delivered: HashSet<String>,
}

impl flurry::Process for BroadcastProcess {
    fn on_message(&mut self, from: flurry::ProcessId, msg: String) {
        if self.delivered.contains(&msg) {
            return;
        }
        for to in self.others.iter() {
            if *to != from {
                flurry::send(*to, msg.clone());
            }
        }
        self.delivered.insert(msg.clone());
        flurry::send_local(msg.clone());
    }

    fn on_local_message(&mut self, msg: &str) {
        for to in self.others.iter() {
            flurry::send(*to, msg.to_string());
        }
        self.delivered.insert(msg.to_string());
        flurry::send_local(msg.to_string());
    }
}
