use std::{sync::Arc, time::Duration};

use rust_extensions::{
    events_loop::{EventsLoop, EventsLoopLogger},
    ApplicationStates, MyTimerLogger,
};
use tokio::sync::RwLock;

use crate::{
    persistence::{MessagesPagesRepo, TopicsAndQueuesSnapshotRepo},
    queue_subscribers::SubscriberIdGenerator,
    sessions::SessionsList,
    settings::SettingsModel,
    topics::{Topic, TopicsList},
};

use super::{
    logs::{Logs, SystemProcess},
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
    pub topics_and_queues_repo: Arc<TopicsAndQueuesSnapshotRepo>,
    pub messages_pages_repo: Arc<MessagesPagesRepo>,
    pub logs: Arc<Logs>,
    pub sessions: SessionsList,
    pub process_id: String,
    pub subscriber_id_generator: SubscriberIdGenerator,

    pub empty_queue_gc_timeout: Duration,
    pub prometheus: PrometheusMetrics,

    pub delivery_timeout: Duration,

    pub debug_topic_and_queue: RwLock<Option<DebugTopicAndQueue>>,

    pub auto_create_topic_on_publish: bool,
    pub auto_create_topic_on_subscribe: bool,

    pub immediatly_persist_event_loop: EventsLoop<Arc<Topic>>,
}

impl AppContext {
    pub fn new(settings: &SettingsModel) -> Self {
        let logs = Arc::new(Logs::new());

        let topics_and_queues_repo = settings.create_topics_and_queues_snapshot_repo();
        let messages_pages_repo = settings.create_messages_pages_repo();
        Self {
            states: GlobalStates::new(),
            topic_list: TopicsList::new(),
            max_delivery_size: settings.max_delivery_size,
            topics_and_queues_repo: Arc::new(topics_and_queues_repo),
            messages_pages_repo: Arc::new(messages_pages_repo),
            logs,
            sessions: SessionsList::new(),
            process_id: uuid::Uuid::new_v4().to_string(),
            empty_queue_gc_timeout: settings.queue_gc_timeout,
            subscriber_id_generator: SubscriberIdGenerator::new(),
            prometheus: PrometheusMetrics::new(),

            delivery_timeout: if let Some(delivery_timeout) = settings.delivery_timeout {
                delivery_timeout
            } else {
                Duration::from_secs(30)
            },
            debug_topic_and_queue: RwLock::new(None),
            auto_create_topic_on_publish: settings.auto_create_topic_on_publish,
            auto_create_topic_on_subscribe: settings.auto_create_topic_on_subscribe,
            immediatly_persist_event_loop: EventsLoop::new("ImmediatelyPersist".to_string()),
        }
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

    pub fn get_max_delivery_size(&self) -> usize {
        self.max_delivery_size
    }
}

impl ApplicationStates for AppContext {
    fn is_initialized(&self) -> bool {
        self.states.is_initialized()
    }

    fn is_shutting_down(&self) -> bool {
        self.states.is_shutting_down()
    }
}

impl MyTimerLogger for AppContext {
    fn write_info(&self, timer_id: String, message: String) {
        self.logs
            .add_info(None, SystemProcess::Timer, timer_id, message);
    }
    fn write_error(&self, timer_id: String, message: String) {
        self.logs
            .add_fatal_error(SystemProcess::Timer, timer_id, message);
    }
}

impl EventsLoopLogger for AppContext {
    fn write_info(&self, timer_id: String, message: String) {
        self.logs
            .add_info(None, SystemProcess::Timer, timer_id, message);
    }
    fn write_error(&self, timer_id: String, message: String) {
        self.logs
            .add_fatal_error(SystemProcess::Timer, timer_id, message);
    }
}
