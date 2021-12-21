use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct TopicPublisherJsonModel {
    #[serde(rename = "messageId")]
    pub session_id: i64,
    pub active: u8,
}
