use std::collections::HashMap;

use my_service_bus_shared::messages_page::MessagesPagesCache;
use my_service_bus_shared::page_id::get_page_id;
use my_service_bus_shared::{
    queue_with_intervals::QueueWithIntervals, MessageId, MySbMessageContent,
};
use my_service_bus_tcp_shared::MessageToPublishTcpContract;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::queue_subscribers::QueueSubscriber;
use crate::queues::delivery_iterator::DeliveryIterator;
use crate::queues::{TopicQueue, TopicQueuesList};
use crate::sessions::SessionId;

use super::TopicMetrics;
const BADGE_HIGHLIGHT_TIMOUT: u8 = 2;

pub struct TopicData {
    pub topic_id: String,
    pub message_id: MessageId,
    pub queues: TopicQueuesList,
    pub metrics: TopicMetrics,
    pub pages: MessagesPagesCache,
    pub publishers: HashMap<SessionId, u8>,
}

impl TopicData {
    pub fn new(topic_id: String, message_id: MessageId) -> Self {
        Self {
            topic_id,
            message_id,
            queues: TopicQueuesList::new(),
            metrics: TopicMetrics::new(),
            pages: MessagesPagesCache::new(),
            publishers: HashMap::new(),
        }
    }

    #[inline]
    pub fn set_publisher_as_active(&mut self, session_id: SessionId) {
        self.publishers.insert(session_id, BADGE_HIGHLIGHT_TIMOUT);
    }

    pub fn publish_messages(
        &mut self,
        session_id: SessionId,
        messages: Vec<MessageToPublishTcpContract>,
    ) {
        self.set_publisher_as_active(session_id);

        let mut ids = QueueWithIntervals::new();

        for msg in messages {
            let message = MySbMessageContent {
                id: self.message_id,
                content: msg.content,
                time: DateTimeAsMicroseconds::now(),
                headers: msg.headers,
            };

            ids.enqueue(message.id);

            let page_id = get_page_id(message.id);

            self.pages
                .get_or_create_page_mut(page_id)
                .new_message(message);

            self.message_id = self.message_id + 1;
        }

        for topic_queue in self.queues.get_all_mut() {
            topic_queue.enqueue_messages(&ids);
        }
    }

    pub fn get_active_pages(&self) -> HashMap<i64, i64> {
        let mut result: HashMap<i64, i64> = HashMap::new();

        let last_message_page_id = get_page_id(self.message_id);

        result.insert(last_message_page_id, last_message_page_id);

        for queue in self.queues.get_all() {
            if let Some(topic_min_msg_id) = queue.get_min_msg_id() {
                let last_min_page_id = get_page_id(topic_min_msg_id);

                if !result.contains_key(&last_min_page_id) {
                    result.insert(last_min_page_id, last_min_page_id);
                }
            }
        }

        result
    }

    pub fn get_min_message_id(&self) -> MessageId {
        match self.queues.get_min_message_id() {
            Some(queue_min_message_id) => queue_min_message_id,
            None => self.message_id,
        }
    }

    pub fn one_second_tick(&mut self) {
        for value in self.publishers.values_mut() {
            if *value > 0 {
                *value -= 1;
            }
        }

        self.metrics
            .one_second_tick(self.pages.get_persist_queue_size());

        self.queues.one_second_tick();
    }

    pub fn disconnect(
        &mut self,
        session_id: SessionId,
    ) -> Option<Vec<(&mut TopicQueue, QueueSubscriber)>> {
        self.publishers.remove(&session_id);

        self.queues.remove_subscribers_by_session_id(session_id)
    }

    pub async fn get_topic_data_size(&self) -> usize {
        let mut result = 0;

        for page in self.pages.get_pages() {
            result += page.size
        }

        result
    }

    pub fn get_delivery_iterator<'s>(
        &'s mut self,
        max_size: usize,
    ) -> Option<DeliveryIterator<'s>> {
        for topic_queue in self.queues.get_all_mut() {
            if topic_queue.queue.len() == 0 {
                continue;
            }

            let subscriber = topic_queue
                .subscribers
                .get_and_rent_next_subscriber_ready_to_deliver();

            if subscriber.is_none() {
                continue;
            }

            let result = DeliveryIterator::new(
                self.topic_id.as_str(),
                topic_queue.queue_id.as_str(),
                &mut self.pages,
                &mut topic_queue.queue,
                &mut topic_queue.delivery_attempts,
                subscriber.unwrap(),
                max_size,
            );

            return Some(result);
        }

        return None;
    }
}
