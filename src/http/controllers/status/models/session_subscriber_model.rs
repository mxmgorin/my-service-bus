use serde::{Deserialize, Serialize};

use crate::queue_subscribers::SubscriberMetrics;

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionSubscriberJsonContract {
    pub id: i64,
    #[serde(rename = "topicId")]
    pub topic_id: String,
    #[serde(rename = "queueId")]
    pub queue_id: String,
    pub active: u8,
    #[serde(rename = "deliveryHistory")]
    pub delivery_history: Vec<i32>,
    #[serde(rename = "deliveryMode")]
    pub delivery_mode: u8,
}

impl SessionSubscriberJsonContract {
    pub fn new(metrics: &SubscriberMetrics) -> Self {
        Self {
            id: metrics.subscriber_id,
            topic_id: metrics.topic.topic_id.to_string(),
            queue_id: metrics.queue.queue_id.to_string(),
            active: metrics.active,
            delivery_history: metrics.delivery_history.get(),
            delivery_mode: metrics.delivery_mode,
        }
    }
}
