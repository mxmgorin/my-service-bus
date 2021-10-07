use std::{collections::HashMap, sync::Arc};

use crate::queues::TopicQueue;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct QueuesJsonResult {
    pub queues: Vec<QueueJsonContract>,
    #[serde(rename = "snapshotId")]
    pub snapshot_id: usize,
}

impl QueuesJsonResult {
    pub async fn new(
        queues: &HashMap<String, (usize, Vec<Arc<TopicQueue>>)>,
    ) -> HashMap<String, Self> {
        let mut result = HashMap::new();

        for (topic_id, (snapshot_id, topic_queues)) in queues {
            let mut queues = Vec::new();

            for topic_queue in topic_queues {
                let monitoring_data = topic_queue.metrics.get().await;

                queues.push(QueueJsonContract {
                    id: monitoring_data.id,
                    queue_type: monitoring_data.queue_type.into_u8(),
                    size: monitoring_data.size,
                    data: monitoring_data
                        .queue
                        .iter()
                        .map(|itm| QueueIndex {
                            from_id: itm.from_id,
                            to_id: itm.to_id,
                        })
                        .collect(),
                });
            }

            result.insert(
                topic_id.to_string(),
                QueuesJsonResult {
                    queues,
                    snapshot_id: *snapshot_id,
                },
            );
        }

        result
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QueueJsonContract {
    id: String,
    #[serde(rename = "queueType")]
    queue_type: u8,
    size: i64,
    data: Vec<QueueIndex>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QueueIndex {
    #[serde(rename = "fromId")]
    pub from_id: i64,
    #[serde(rename = "toId")]
    pub to_id: i64,
}
