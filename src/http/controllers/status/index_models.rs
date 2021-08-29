use std::collections::HashMap;

use crate::app::AppContext;

use serde::{Deserialize, Serialize};
use sysinfo::SystemExt;

use super::models::{
    queue_model::QueuesJsonResult, session_model::SessionsJsonResult, topic_model::TopicsJsonResult,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct SystemStatusModel {
    usedmem: u64,
    totalmem: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StatusJsonResult {
    pub topics: TopicsJsonResult,
    pub queues: HashMap<String, QueuesJsonResult>,
    pub sessions: SessionsJsonResult,
    pub system: SystemStatusModel,
}

impl StatusJsonResult {
    pub async fn new(app: &AppContext) -> Self {
        let topics = app.topic_list.get_all().await;

        let mut sys_info = sysinfo::System::new_all();

        sys_info.refresh_all();

        Self {
            topics: TopicsJsonResult::new(app, topics.as_slice()).await,
            queues: QueuesJsonResult::new(topics.as_slice()).await,
            sessions: SessionsJsonResult::new(app).await,
            system: SystemStatusModel {
                totalmem: sys_info.total_memory(),
                usedmem: sys_info.used_memory(),
            },
        }
    }
}
