use std::{collections::HashMap, sync::Arc};

use crate::topics::Topic;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct QueuesJsonResult {
    pub queues: Vec<QueueJsonContract>,
    #[serde(rename = "snapshotId")]
    pub snapshot_id: usize,
}

impl QueuesJsonResult {
    pub async fn new(topics: &[Arc<Topic>]) -> HashMap<String, Self> {
        let mut result = HashMap::new();

        for topic in topics {
            let (snapshot_id, monitoring_datas) = topic.queues.get_monitoring_data().await;

            let mut queues = Vec::new();

            for mon_data in monitoring_datas {
                queues.push(QueueJsonContract {
                    id: mon_data.id,
                    queue_type: mon_data.queue_type.into_u8(),
                    size: mon_data.size,
                    data: mon_data
                        .queue
                        .iter()
                        .map(|itm| QueueIndex {
                            from_id: itm.from_id,
                            to_id: itm.to_id,
                        })
                        .collect(),
                })
            }

            result.insert(
                topic.topic_id.to_string(),
                QueuesJsonResult {
                    queues,
                    snapshot_id,
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
