use std::{sync::Arc, time::Duration};

use my_service_bus_shared::debug::{LockItem, Locks};
use tokio::sync::Mutex;

use crate::{
    persistence::{MessagesPagesRepo, TopcsAndQueuesSnapshotRepo},
    queue_subscribers::SubscriberIdGenerator,
    sessions::SessionsList,
    settings::SettingsModel,
    topics::TopicsList,
};

use super::{
    logs::Logs, process_id_generator::ProcessIdGenerator, prometheus_metrics::PrometheusMetrics,
    GlobalStates,
};

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

    pub locks: Mutex<Locks>,

    pub process_id_generator: ProcessIdGenerator,

    pub delivery_timeout: Option<Duration>,
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
            locks: Mutex::new(Locks::new()),
            process_id_generator: ProcessIdGenerator::new(),
            delivery_timeout: settings.delivery_timeout,
        }
    }

    pub async fn enter_lock(&self, process_id: i64, lock_name: String) {
        let mut write_access = self.locks.lock().await;
        write_access.new_lock(process_id, lock_name)
    }

    pub async fn exit_lock(&self, id: i64) {
        let mut write_access = self.locks.lock().await;
        write_access.exit(id);
    }

    pub async fn get_locks(&self) -> Vec<LockItem> {
        let read_access = self.locks.lock().await;
        read_access.get_all()
    }
}
