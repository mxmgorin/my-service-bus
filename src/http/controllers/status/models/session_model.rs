use std::{collections::HashMap, sync::Arc};

use crate::{
    app::AppContext, queue_subscribers::SubscriberMetrics, queues::TopicQueue,
    sessions::MyServiceBusSession, tcp::tcp_server::ConnectionId, utils::duration_to_string,
};

use my_service_bus_shared::date_time::DateTimeAsMicroseconds;
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
    pub async fn new(
        subscribers: &[SubscriberMetrics],
        session: &MyServiceBusSession,
        process_id: i64,
    ) -> Self {
        session
            .app
            .enter_lock(process_id, "MySbSession.SessionJsonResult.new".to_string())
            .await;
        let session_read = session.data.read().await;

        let now = DateTimeAsMicroseconds::now();

        let mut subscribers_json = Vec::new();

        for subscriber in subscribers {
            let item = SessionSubscriberJsonContract::new(subscriber);

            subscribers_json.push(item);
        }

        session.app.exit_lock(process_id).await;

        Self {
            id: session.id,
            ip: session.ip.to_string(),
            name: session_read.get_name(),
            version: session_read.get_version(),
            connected: duration_to_string(now.duration_since(session.connected)),
            last_incoming: duration_to_string(
                now.duration_since(session.last_incoming_package.as_date_time()),
            ),
            read_size: session_read.statistic.read_size,
            written_size: session_read.statistic.written_size,
            read_per_sec: session_read.statistic.read_per_sec,
            written_per_sec: session_read.statistic.written_per_sec,
            publishers: session_read.statistic.publishers.clone(),
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
    pub async fn new(
        app: &AppContext,
        process_id: i64,
        queues_as_hashmap: &HashMap<String, (usize, Vec<Arc<TopicQueue>>)>,
    ) -> SessionsJsonResult {
        let subscribers_by_connection_id =
            get_subscribers_by_connection_id(queues_as_hashmap).await;

        let mut items = Vec::new();

        let (snapshot_id, sessions) = app.sessions.get_snapshot().await;

        let empty = [];

        for session in sessions {
            let subscribers = subscribers_by_connection_id.get(&session.id);

            let subscribers = match subscribers {
                Some(subscribers) => subscribers.as_slice(),
                None => &empty,
            };

            let session = SessionJsonResult::new(subscribers, session.as_ref(), process_id).await;
            items.push(session);
        }

        Self {
            snapshot_id: snapshot_id,
            items,
        }
    }
}

async fn get_subscribers_by_connection_id(
    queues: &HashMap<String, (usize, Vec<Arc<TopicQueue>>)>,
) -> HashMap<ConnectionId, Vec<SubscriberMetrics>> {
    let mut result = HashMap::new();

    for (_, topic_queues) in queues.values() {
        for topic_queue in topic_queues {
            let subscriber_metrics = topic_queue.get_all_subscribers_metrics().await;

            for metrics in subscriber_metrics {
                if !result.contains_key(&metrics.connection_id) {
                    result.insert(metrics.connection_id, Vec::new());
                }

                result
                    .get_mut(&metrics.connection_id)
                    .unwrap()
                    .push(metrics);
            }
        }
    }

    result
}
