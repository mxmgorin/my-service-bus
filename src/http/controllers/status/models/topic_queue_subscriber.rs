use serde::{Deserialize, Serialize};

use crate::sessions::SessionId;

#[derive(Serialize, Deserialize, Debug)]
pub struct TopicQueueSubscriberJsonModel {
    #[serde(rename = "id")]
    pub subscriber_id: SessionId,
    #[serde(rename = "sessionId")]
    pub session_id: SessionId,
    #[serde(rename = "queueId")]
    pub queue_id: String,
    pub active: u8,
    #[serde(rename = "deliveryState")]
    pub delivery_state: u8,
    pub history: Vec<i32>,
}
