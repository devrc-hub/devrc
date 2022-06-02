#[derive(Debug, Clone)]
pub struct TaskResult {}

impl TaskResult {
    pub fn new() -> Self {
        TaskResult {}
    }
}

impl Default for TaskResult {
    fn default() -> Self {
        Self::new()
    }
}
