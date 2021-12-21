use std::collections::HashMap;

use crate::app::AppContext;

use serde::{Deserialize, Serialize};
use sysinfo::SystemExt;

use super::models::{
    queue_model::QueuesJsonResult,
    session_model::{SessionJsonResult, SessionsJsonResult},
    topic_model::{TopicJsonContract, TopicsJsonResult},
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
        let mut sys_info = sysinfo::System::new_all();

        sys_info.refresh_all();

        let (snapshot_id, all_topics) = app.topic_list.get_all_with_snapshot_id().await;

        let mut queues = HashMap::new();

        let mut topics = TopicsJsonResult {
            snapshot_id,
            items: Vec::new(),
        };

        let (sessions_snapshot_id, all_sessions) = app.sessions.get_snapshot().await;

        let mut sessions_json_result = SessionsJsonResult {
            snapshot_id: sessions_snapshot_id,
            items: Vec::new(),
        };

        let mut sessions_as_hashmap = HashMap::new();

        for session in &all_sessions {
            let session_json_model = SessionJsonResult::new(session.as_ref()).await;
            sessions_json_result.items.push(session_json_model);
            sessions_as_hashmap.insert(session.id, session.clone());
        }

        for topic in all_topics {
            let topic_data = topic.data.lock().await;
            queues.insert(
                topic_data.topic_id.to_string(),
                QueuesJsonResult::new(&topic_data),
            );

            topics.items.push(TopicJsonContract::new(&topic_data));

            for topic_queue in topic_data.queues.get_all() {
                if let Some(subscribers) = topic_queue.subscribers.get_all() {
                    for subscriber in subscribers {
                        if let Some(session) = sessions_as_hashmap.get(&subscriber.session_id) {
                            let json_model = SessionJsonResult::new(session.as_ref()).await;
                            sessions_json_result.items.push(json_model)
                        }
                    }
                }
            }
        }

        Self {
            topics,
            queues,
            sessions: sessions_json_result,
            system: SystemStatusModel {
                totalmem: sys_info.total_memory(),
                usedmem: sys_info.used_memory(),
            },
        }
    }
}
