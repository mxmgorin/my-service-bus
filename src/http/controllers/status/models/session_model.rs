use std::collections::HashMap;

use crate::{
    app::AppContext, date_time::MyDateTime, sessions::MyServiceBusSession,
    utils::duration_to_string,
};

use serde::{Deserialize, Serialize};

use super::session_subscriber_model::SessionSubscriberJsonContract;

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionJsonResult {
    pub id: i64,
    pub name: Option<String>,
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

    pub publishers: HashMap<String, u8>,
    pub subscribers: Vec<SessionSubscriberJsonContract>,
}

impl SessionJsonResult {
    pub async fn new(session: &MyServiceBusSession) -> Self {
        let last_incoming = session.last_incoming_package.get();
        let session_statistic = session.get_statistic().await;

        let now = MyDateTime::utc_now();
        let session_read = session.data.read().await;

        let mut subscribers_json = Vec::new();

        for (_, subscriber) in session_statistic.subscribers {
            let item = SessionSubscriberJsonContract::new(&subscriber);

            subscribers_json.push(item);
        }

        Self {
            id: session.id,
            ip: session.ip.to_string(),
            name: session_read.get_name(),
            version: session_read.get_version(),
            connected: duration_to_string(now.get_duration_from(session.connected)),
            last_incoming: duration_to_string(now.get_duration_from_micros(last_incoming)),
            read_size: session_statistic.read_size,
            written_size: session_statistic.written_size,
            read_per_sec: session_statistic.read_per_sec,
            written_per_sec: session_statistic.written_per_sec,
            publishers: session_statistic.publishers,
            subscribers: subscribers_json,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionsJsonResult {
    #[serde(rename = "snapshotId")]
    pub snapshot_id: usize,
    pub items: Vec<SessionJsonResult>,
}

impl SessionsJsonResult {
    pub async fn new(app: &AppContext) -> SessionsJsonResult {
        let mut items = Vec::new();

        let (snapshot_id, sessions) = app.sessions.get_snapshot().await;

        for session in sessions {
            let session = SessionJsonResult::new(session.as_ref()).await;

            items.push(session);
        }

        Self {
            snapshot_id: snapshot_id,
            items,
        }
    }
}