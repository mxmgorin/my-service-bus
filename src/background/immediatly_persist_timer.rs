use std::sync::Arc;

use rust_extensions::MyTimerTick;

use crate::app::AppContext;

pub struct ImmediatlyPersistTimer {
    app: Arc<AppContext>,
}

impl ImmediatlyPersistTimer {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl MyTimerTick for ImmediatlyPersistTimer {
    async fn tick(&self) {
        let topics = self.app.topic_list.get_topics_to_immediatly_persist().await;

        if let Some(topics) = topics {
            for topic in topics {
                crate::operations::save_messages_for_topic(&self.app, &topic).await;
            }
        }
    }
}
