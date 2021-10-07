use serde::{Deserialize, Serialize};

use crate::queues::subscribers::SubscriberMetrics;

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
}

impl SessionSubscriberJsonContract {
    pub fn new(subscriber: &SubscriberMetrics) -> Self {
        Self {
            id: subscriber.subscriber_id,
            topic_id: subscriber.topic.topic_id.to_string(),
            queue_id: subscriber.queue.queue_id.to_string(),
            active: subscriber.active,
            delivery_history: subscriber.delivery_history.get(),
        }
    }
}
