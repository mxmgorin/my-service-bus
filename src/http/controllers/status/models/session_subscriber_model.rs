use crate::sessions::MySbSessionSubscriberData;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionSubscriberJsonContract {
    #[serde(rename = "topicId")]
    pub topic_id: String,
    #[serde(rename = "queueId")]
    pub queue_id: String,
    pub active: u8,
    #[serde(rename = "deliveryHistory")]
    pub delivery_history: Vec<i32>,
}

impl SessionSubscriberJsonContract {
    pub fn new(subscriber: &MySbSessionSubscriberData) -> Self {
        Self {
            topic_id: subscriber.topic_id.to_string(),
            queue_id: subscriber.queue_id.to_string(),
            active: subscriber.active,
            delivery_history: subscriber.metrics.get(),
        }
    }
}
