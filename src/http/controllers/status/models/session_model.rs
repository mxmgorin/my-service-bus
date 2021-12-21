use crate::sessions::MyServiceBusSession;

use rust_extensions::date_time::DateTimeAsMicroseconds;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionJsonResult {
    pub id: i64,
    pub name: String,
    pub ip: String,
    pub version: Option<String>,
    pub connected: String,
    #[serde(rename = "lastIncoming")]
    pub last_incoming: String,
    #[serde(rename = "readSize")]
    pub read_size: usize,
    #[serde(rename = "writtenSize")]
    pub written_size: usize,
    #[serde(rename = "readPerSec")]
    pub read_per_sec: usize,
    #[serde(rename = "writtenPerSec")]
    pub written_per_sec: usize,
}

impl SessionJsonResult {
    pub async fn new(session: &MyServiceBusSession) -> Self {
        let now = DateTimeAsMicroseconds::now();

        let session_metrics_data = session.get_metrics().await;

        let name = if let Some(name) = session_metrics_data.name {
            name
        } else {
            "???".to_string()
        };

        Self {
            id: session_metrics_data.id,
            ip: session_metrics_data.ip,
            name: format!("{}[{}]", name, session_metrics_data.protocol_version),
            version: session_metrics_data.version,
            connected: rust_extensions::duration_utils::duration_to_string(
                now.duration_since(session.connected),
            ),
            last_incoming: rust_extensions::duration_utils::duration_to_string(
                now.duration_since(session.last_incoming_package.as_date_time()),
            ),
            read_size: session_metrics_data.metrics.read_size,
            written_size: session_metrics_data.metrics.written_size,
            read_per_sec: session_metrics_data.metrics.read_per_sec,
            written_per_sec: session_metrics_data.metrics.written_per_sec,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionsJsonResult {
    #[serde(rename = "snapshotId")]
    pub snapshot_id: usize,
    pub items: Vec<SessionJsonResult>,
}
