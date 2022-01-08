use serde::{Deserialize, Serialize};

use crate::sessions::SessionId;

#[derive(Serialize, Deserialize, Debug)]
pub struct TopicPublisherJsonModel {
    #[serde(rename = "sessionId")]
    pub session_id: SessionId,
    pub active: u8,
}
