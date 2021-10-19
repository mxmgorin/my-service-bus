use my_service_bus_shared::messages_page::MessagesPagesCache;
use my_service_bus_shared::page_id::{get_page_id, PageId};
use my_service_bus_shared::queue_with_intervals::QueueWithIntervals;
use my_service_bus_shared::MySbMessageContent;
use my_service_bus_shared::{queue::TopicQueueType, MessageId};
use rust_extensions::date_time::DateTimeAsMicroseconds;
use tokio::sync::RwLock;

use crate::queues::TopicQueue;
use crate::queues::TopicQueuesList;
use std::collections::HashMap;
use std::{collections::VecDeque, sync::Arc};

use super::topic_snapshot::TopicSnapshot;
use super::TopicMetrics;

pub struct TopicData {
    message_id: MessageId,
}

pub struct GetMinMessageIdResult {
    pub topic_message_id: MessageId,
    pub queue_min_message_id: Option<MessageId>,
}

pub struct Topic {
    pub topic_id: String,
    pub data: RwLock<TopicData>,
    pub metrics: TopicMetrics,
    pub messages: MessagesPagesCache,
    pub queues: TopicQueuesList,
}

impl Topic {
    pub fn new(topic_id: &str, message_id: MessageId) -> Self {
        Self {
            topic_id: topic_id.to_string(),
            data: RwLock::new(TopicData { message_id }),
            queues: TopicQueuesList::new(),
            metrics: TopicMetrics::new(),
            messages: MessagesPagesCache::new(),
        }
    }

    pub async fn publish_messages(&self, messages: Vec<Vec<u8>>) -> VecDeque<MySbMessageContent> {
        self.metrics.update_topic_metrics(messages.len()).await;

        let mut result = VecDeque::new();

        let mut topic_write_access = self.data.write().await;

        for content in messages {
            let message = MySbMessageContent {
                id: topic_write_access.message_id,
                content,
                time: DateTimeAsMicroseconds::now(),
            };
            result.push_back(message);
            topic_write_access.message_id = topic_write_access.message_id + 1;
        }

        result
    }

    pub async fn get_message_id(&self) -> MessageId {
        self.data.read().await.message_id
    }

    pub async fn restore_queue(
        &self,
        queue_id: &str,
        queue_type: TopicQueueType,
        queue: QueueWithIntervals,
    ) -> Arc<TopicQueue> {
        let result = self
            .queues
            .restore(self.topic_id.as_str(), queue_id, queue_type, queue)
            .await;

        result
    }

    pub async fn get_queue(&self, queue_id: &str) -> Option<Arc<TopicQueue>> {
        self.queues.get(queue_id).await
    }

    pub async fn get_all_queues(&self) -> Vec<Arc<TopicQueue>> {
        self.queues.get_queues().await
    }

    pub async fn get_all_queues_with_snapshot_id(&self) -> (usize, Vec<Arc<TopicQueue>>) {
        self.queues.get_queues_with_snapshot_id().await
    }

    pub async fn delete_queue(&self, queue_id: &str) -> Option<Arc<TopicQueue>> {
        self.queues.delete_queue(queue_id).await
    }

    pub async fn get_snapshot_to_persist(&self) -> TopicSnapshot {
        return TopicSnapshot {
            topic_id: self.topic_id.clone(),
            message_id: self.get_message_id().await,
            queues: self.queues.get_snapshot_to_persist().await,
        };
    }

    pub async fn get_current_page(&self) -> PageId {
        let read_access = self.data.read().await;

        get_page_id(read_access.message_id)
    }

    pub async fn get_active_pages(&self) -> HashMap<i64, i64> {
        let mut result: HashMap<i64, i64> = HashMap::new();

        let last_message_id = self.get_message_id().await;

        let last_message_page_id = get_page_id(last_message_id);

        result.insert(last_message_page_id, last_message_page_id);

        for queue in self.get_all_queues().await {
            if let Some(topic_min_msg_id) = queue.get_min_msg_id().await {
                let last_min_page_id = get_page_id(topic_min_msg_id);

                if !result.contains_key(&last_min_page_id) {
                    result.insert(last_min_page_id, last_min_page_id);
                }
            }
        }

        result
    }

    pub async fn one_second_tick(&self) {
        let persist_queue_size = self.messages.get_persist_queue_size().await;
        self.metrics.one_second_tick(persist_queue_size).await;
        self.queues.one_second_tick().await;
    }

    pub async fn get_min_message_id(&self) -> GetMinMessageIdResult {
        GetMinMessageIdResult {
            topic_message_id: self.get_message_id().await,
            queue_min_message_id: self.queues.get_min_message_id().await,
        }
    }
}
