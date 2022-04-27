use my_http_server_swagger::MyHttpObjectStructure;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct TopicPublisherJsonModel {
    #[serde(rename = "sessionId")]
    pub session_id: i64,
    pub active: u8,
}
