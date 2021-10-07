use std::sync::Arc;

use my_service_bus_shared::date_time::DateTimeAsMicroseconds;

use crate::{
    messages_bucket::MessagesBucket, queues::TopicQueue, sessions::MyServiceBusSession,
    tcp::tcp_server::ConnectionId, topics::Topic,
};

use super::{SubscriberId, SubscriberMetrics};

pub struct QueueSubscriber {
    pub topic: Arc<Topic>,
    pub queue: Arc<TopicQueue>,
    pub subscribed: DateTimeAsMicroseconds,
    pub metrics: SubscriberMetrics,
    pub messages_on_delivery: Option<MessagesBucket>,

    pub id: SubscriberId,
    pub session: Arc<MyServiceBusSession>,
    pub rented: bool,
}

impl QueueSubscriber {
    pub fn new(
        id: SubscriberId,
        connection_id: ConnectionId,
        topic: Arc<Topic>,
        queue: Arc<TopicQueue>,
        session: Arc<MyServiceBusSession>,
    ) -> Self {
        Self {
            topic: topic.clone(),
            queue: queue.clone(),
            subscribed: DateTimeAsMicroseconds::now(),
            metrics: SubscriberMetrics::new(id, connection_id, topic, queue),
            messages_on_delivery: None,
            rented: false,
            session,
            id,
        }
    }

    pub fn reset_delivery(&mut self) -> Option<MessagesBucket> {
        let mut result = None;
        std::mem::swap(&mut result, &mut self.messages_on_delivery);
        self.rented = false;
        return result;
    }

    pub fn get_messages_amount_on_delivery(&self) -> usize {
        if let Some(messages) = &self.messages_on_delivery {
            return messages.messages_count();
        }

        return 0;
    }
}
