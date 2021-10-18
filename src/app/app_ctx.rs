use std::{sync::Arc, time::Duration};

use tokio::sync::{mpsc::UnboundedSender, RwLock};

use crate::{
    persistence::{MessagesPagesRepo, TopcsAndQueuesSnapshotRepo},
    queue_subscribers::SubscriberIdGenerator,
    sessions::SessionsList,
    settings::SettingsModel,
    topics::TopicsList,
};

use super::{
    locks_registry::{LockEvent, LocksRegistry},
    logs::Logs,
    process_id_generator::ProcessIdGenerator,
    prometheus_metrics::PrometheusMetrics,
    GlobalStates,
};

pub const APP_VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(Clone)]
pub struct DebugTopicAndQueue {
    pub topic_id: String,
    pub queue_id: String,
}

pub struct AppContext {
    pub states: GlobalStates,
    pub topic_list: TopicsList,
    pub max_delivery_size: usize,
    pub topics_and_queues_repo: TopcsAndQueuesSnapshotRepo,
    pub messages_pages_repo: MessagesPagesRepo,
    pub logs: Arc<Logs>,
    pub sessions: SessionsList,
    pub process_id: String,
    pub subscriber_id_generator: SubscriberIdGenerator,

    pub empty_queue_gc_timeout: Duration,
    pub prometheus: PrometheusMetrics,

    pub process_id_generator: ProcessIdGenerator,

    pub delivery_timeout: Option<Duration>,

    pub locks: Arc<LocksRegistry>,

    pub debug_topic_and_queue: RwLock<Option<DebugTopicAndQueue>>,

    pub auto_create_topic: bool,
}

impl AppContext {
    pub fn new(settings: &SettingsModel, locks_sender: UnboundedSender<LockEvent>) -> Self {
        let logs = Arc::new(Logs::new());
        Self {
            states: GlobalStates::new(),
            topic_list: TopicsList::new(),
            max_delivery_size: settings.max_delivery_size,
            topics_and_queues_repo: TopcsAndQueuesSnapshotRepo::new(settings),
            messages_pages_repo: MessagesPagesRepo::new(settings),
            logs,
            sessions: SessionsList::new(),
            process_id: uuid::Uuid::new_v4().to_string(),
            empty_queue_gc_timeout: settings.queue_gc_timeout,
            subscriber_id_generator: SubscriberIdGenerator::new(),
            prometheus: PrometheusMetrics::new(),

            process_id_generator: ProcessIdGenerator::new(),
            delivery_timeout: settings.delivery_timeout,
            locks: Arc::new(LocksRegistry::new(locks_sender)),
            debug_topic_and_queue: RwLock::new(None),
            auto_create_topic: settings.auto_create_topic,
        }
    }

    pub async fn get_debug_topic_and_queue(&self) -> Option<DebugTopicAndQueue> {
        let read_access = self.debug_topic_and_queue.read().await;
        let result = read_access.as_ref()?;
        return Some(result.clone());
    }

    pub async fn set_debug_topic_and_queue(&self, topic_id: &str, queue_id: &str) {
        let mut write_access = self.debug_topic_and_queue.write().await;

        *write_access = Some(DebugTopicAndQueue {
            topic_id: topic_id.to_string(),
            queue_id: queue_id.to_string(),
        })
    }

    pub async fn disable_debug_topic_and_queue(&self) {
        let mut write_access = self.debug_topic_and_queue.write().await;

        *write_access = None;
    }
}
