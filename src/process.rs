pub type ProcessId = usize;

/// Represents requirements for the user process.
pub trait Process {
    fn on_message(&mut self, from: ProcessId, msg: &str);

    fn on_local_message(&mut self, msg: &str);
}
