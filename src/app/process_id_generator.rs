use tokio::sync::Mutex;

pub struct ProcessIdGenerator {
    current: Mutex<i64>,
}

impl ProcessIdGenerator {
    pub fn new() -> Self {
        Self {
            current: Mutex::new(0),
        }
    }

    pub async fn get_process_id(&self) -> i64 {
        let mut write_access = self.current.lock().await;

        *write_access += 1;

        *write_access
    }
}
