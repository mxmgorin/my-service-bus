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
        let all_topics = app.topic_list.get_all().await;

        let mut sys_info = sysinfo::System::new_all();

        sys_info.refresh_all();

        println!("Reading Topics");
        let topics = TopicsJsonResult::new(app, all_topics.as_slice()).await;

        println!("Reading Queues");
        let queues = QueuesJsonResult::new(all_topics.as_slice()).await;

        println!("Reading Sessions");
        let sessions = SessionsJsonResult::new(app).await;

        println!("Compling status model");

        Self {
            topics,
            queues,
            sessions,
            system: SystemStatusModel {
                totalmem: sys_info.total_memory(),
                usedmem: sys_info.used_memory(),
            },
        }
    }
}
