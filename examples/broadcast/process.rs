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
                let to = *to;
                let msg = msg.clone();
                flurry::spawn(async move { flurry::send(to, msg).await });
            }
        }
        self.delivered.insert(msg.clone());
        flurry::send_local(msg.clone());
    }

    fn on_local_message(&mut self, msg: &str) {
        for to in self.others.iter() {
            let to = *to;
            let msg = msg.to_string();
            flurry::spawn(async move { flurry::send(to, msg).await });
        }
        self.delivered.insert(msg.to_string());
        flurry::send_local(msg.to_string());
    }
}
