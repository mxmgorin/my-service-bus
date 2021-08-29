use std::{sync::Arc, time::Duration};

use crate::{
    persistence::{MessagesPagesRepo, TopcsAndQueuesSnapshotRepo},
    sessions::SessionsList,
    settings::SettingsModel,
    subscribers::SubscriberIdGenerator,
    topics::TopicsList,
};

use super::{logs::Logs, prometheus_metrics::PrometheusMetrics, GlobalStates};

pub const APP_VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub const TEST_QUEUE: &str = "test-queue";

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
}

impl AppContext {
    pub fn new(settings: &SettingsModel) -> Self {
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
        }
    }
}