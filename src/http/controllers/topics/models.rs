use crate::topics::Topic;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(transparent)]
pub struct JsonTopicsResult {
    pub items: Vec<JsonTopicResult>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonTopicResult {
    pub id: String,
    #[serde(rename = "messageId")]
    pub message_id: i64,
}

impl JsonTopicResult {
    pub async fn new(topic: &Topic) -> Self {
        Self {
            id: topic.topic_id.to_string(),
            message_id: topic.get_message_id().await,
        }
    }
}
