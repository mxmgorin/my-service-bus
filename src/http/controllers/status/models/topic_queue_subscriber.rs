use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct TopicQueueSubscriberJsonModel {
    #[serde(rename = "id")]
    pub subscriber_id: i64,
    #[serde(rename = "sessionId")]
    pub session_id: i32,
    #[serde(rename = "queueId")]
    pub queue_id: String,
    pub active: u8,
    #[serde(rename = "deliveryState")]
    pub delivery_state: u8,
    pub history: Vec<i32>,
}
