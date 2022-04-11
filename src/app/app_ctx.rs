use std::{sync::Arc, time::Duration};

use rust_extensions::ApplicationStates;
use tokio::sync::RwLock;

use crate::{
    operations::delivery::DeliveryDependecies,
    persistence::{MessagesPagesGrpcRepo, TopcsAndQueuesSnapshotRepo},
    queue_subscribers::SubscriberIdGenerator,
    sessions::{MyServiceBusSession, SessionsList},
    settings::SettingsModel,
    topics::TopicsList,
};

use super::{logs::Logs, prometheus_metrics::PrometheusMetrics, GlobalStates};

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
    pub messages_pages_repo: MessagesPagesGrpcRepo,
    pub logs: Arc<Logs>,
    pub sessions: SessionsList,
    pub process_id: String,
    pub subscriber_id_generator: SubscriberIdGenerator,

    pub empty_queue_gc_timeout: Duration,
    pub prometheus: PrometheusMetrics,

    pub delivery_timeout: Option<Duration>,

    pub debug_topic_and_queue: RwLock<Option<DebugTopicAndQueue>>,

    pub auto_create_topic_on_publish: bool,
    pub auto_create_topic_on_subscribe: bool,
}

impl AppContext {
    pub fn new(settings: &SettingsModel) -> Self {
        let logs = Arc::new(Logs::new());
        Self {
            states: GlobalStates::new(),
            topic_list: TopicsList::new(),
            max_delivery_size: settings.max_delivery_size,
            topics_and_queues_repo: TopcsAndQueuesSnapshotRepo::new(settings),
            messages_pages_repo: MessagesPagesGrpcRepo::new(settings),
            logs,
            sessions: SessionsList::new(),
            process_id: uuid::Uuid::new_v4().to_string(),
            empty_queue_gc_timeout: settings.queue_gc_timeout,
            subscriber_id_generator: SubscriberIdGenerator::new(),
            prometheus: PrometheusMetrics::new(),

            delivery_timeout: settings.delivery_timeout,
            debug_topic_and_queue: RwLock::new(None),
            auto_create_topic_on_publish: settings.auto_create_topic_on_publish,
            auto_create_topic_on_subscribe: settings.auto_create_topic_on_subscribe,
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
}

impl DeliveryDependecies for Arc<AppContext> {
    fn get_max_delivery_size(&self) -> usize {
        return self.max_delivery_size;
    }

    fn send_package(
        &self,
        session: Arc<MyServiceBusSession>,
        tcp_packet: my_service_bus_tcp_shared::TcpContract,
    ) {
        tokio::spawn(async move {
            match &session.connection {
                crate::sessions::SessionConnection::Tcp(data) => {
                    data.connection.send(tcp_packet).await;
                }
                #[cfg(test)]
                crate::sessions::SessionConnection::Test(_) => {
                    panic!("Test connection is not supported")
                }
                crate::sessions::SessionConnection::Http(_) => todo!("Not suppored yet"),
            }
        });
    }

    fn load_page(
        &self,
        topic: Arc<crate::topics::Topic>,
        page_id: my_service_bus_shared::page_id::PageId,
    ) {
        let app = self.clone();
        tokio::spawn(async move {
            crate::operations::page_loader::load_full_page_to_cache(
                topic.clone(),
                &app.messages_pages_repo,
                Some(app.logs.as_ref()),
                page_id,
            )
            .await;
            let mut topic_data = topic.get_access("app.load_page").await;
            crate::operations::delivery::try_to_deliver(&app, &topic, &mut topic_data);
        });
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
