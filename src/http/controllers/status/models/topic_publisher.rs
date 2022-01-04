use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct TopicPublisherJsonModel {
    #[serde(rename = "sessionId")]
    pub session_id: i32,
    pub active: u8,
}
