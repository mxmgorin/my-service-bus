use std::{collections::HashMap, sync::Arc};

use crate::{app::AppContext, queues::TopicQueue, topics::Topic};

use serde::{Deserialize, Serialize};
//use sysinfo::SystemExt;

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
    pub async fn new(app: &AppContext, process_id: i64) -> Self {
        let all_topics = app.topic_list.get_all().await;

        let queues_as_hashmap = get_queues_as_hashmap(&all_topics).await;

        // let mut sys_info = sysinfo::System::new_all();

        //sys_info.refresh_all();

        let topics = TopicsJsonResult::new(app, &all_topics).await;

        let queues = QueuesJsonResult::new(&queues_as_hashmap).await;

        let sessions = SessionsJsonResult::new(app, process_id, &queues_as_hashmap).await;

        Self {
            topics,
            queues,
            sessions,
            system: SystemStatusModel {
                totalmem: 0, //sys_info.total_memory(),
                usedmem: 0,  //sys_info.used_memory(),
            },
        }
    }
}

async fn get_queues_as_hashmap(
    topics: &[Arc<Topic>],
) -> HashMap<String, (usize, Vec<Arc<TopicQueue>>)> {
    let mut result = HashMap::new();

    for topic in topics {
        let with_snapshot_id = topic.get_all_queues_with_snapshot_id().await;
        result.insert(topic.topic_id.to_string(), with_snapshot_id);
    }

    result
}
