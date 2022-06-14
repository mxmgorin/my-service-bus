use std::sync::Arc;

use rust_extensions::MyTimerTick;

use crate::app::AppContext;

pub struct PersistTopicsAndQueuesTimer {
    app: Arc<AppContext>,
}

impl PersistTopicsAndQueuesTimer {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl MyTimerTick for PersistTopicsAndQueuesTimer {
    async fn tick(&self) {
        crate::operations::persist_topics_and_queues(&self.app).await;
    }
}
