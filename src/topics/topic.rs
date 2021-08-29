use my_service_bus_shared::page_id::{get_page_id, PageId};
use my_service_bus_shared::queue_with_intervals::QueueWithIntervals;
use my_service_bus_shared::MessageId;
use tokio::sync::RwLock;

use crate::date_time::MyDateTime;
use crate::message_pages::MessagesPagesCache;
use crate::messages::MySbMessageContent;
use crate::queues::TopicQueuesList;
use crate::queues::{TopicQueue, TopicQueueType};
use std::collections::HashMap;
use std::{collections::VecDeque, sync::Arc};

use super::topic_snapshot::TopicSnapshot;
use super::TopicMetrics;

pub struct TopicData {
    message_id: MessageId,
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
                time: MyDateTime::utc_now(),
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

    pub async fn delete_queue(&self, queue_id: &str) -> Option<Arc<TopicQueue>> {
        self.queues.delete_queue(queue_id).await
    }

    pub async fn get_snapshot(&self) -> TopicSnapshot {
        return TopicSnapshot {
            topic_id: self.topic_id.clone(),
            message_id: self.get_message_id().await,
            queues: self.queues.get_snapshot().await,
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
}
