use std::sync::Arc;

use my_service_bus_shared::date_time::DateTimeAsMicroseconds;

use crate::{
    messages_bucket::MessagesBucket, queues::TopicQueue, sessions::MyServiceBusSession,
    tcp::tcp_server::ConnectionId, topics::Topic,
};

use super::{SubscriberId, SubscriberMetrics};

pub enum QueueSubscriberDeliveryState {
    ReadyToDeliver,
    Rented,
    OnDelivery(MessagesBucket),
}

impl QueueSubscriberDeliveryState {
    pub fn to_string(&self) -> &str {
        match self {
            QueueSubscriberDeliveryState::ReadyToDeliver => "ReadyToDeliver",
            QueueSubscriberDeliveryState::Rented => "Rented",
            QueueSubscriberDeliveryState::OnDelivery(_) => "OnDelivery",
        }
    }
}

pub struct QueueSubscriber {
    pub topic: Arc<Topic>,
    pub queue: Arc<TopicQueue>,
    pub subscribed: DateTimeAsMicroseconds,
    pub metrics: SubscriberMetrics,
    pub delivery_state: QueueSubscriberDeliveryState,

    pub id: SubscriberId,
    pub session: Arc<MyServiceBusSession>,
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
            delivery_state: QueueSubscriberDeliveryState::ReadyToDeliver,
            session,
            id,
        }
    }

    pub fn rent_me(&mut self) -> bool {
        if let QueueSubscriberDeliveryState::ReadyToDeliver = &self.delivery_state {
            self.delivery_state = QueueSubscriberDeliveryState::Rented;
            return true;
        }

        return false;
    }

    pub fn cancel_the_rent(&mut self) {
        if let QueueSubscriberDeliveryState::Rented = &self.delivery_state {
            self.delivery_state = QueueSubscriberDeliveryState::Rented;
            return;
        }

        panic!(
            "Can not cancel the rented state. Subscriber is in the {} state",
            self.delivery_state.to_string()
        );
    }

    pub fn reset_delivery(&mut self) -> Option<MessagesBucket> {
        let mut prev_delivery_state = QueueSubscriberDeliveryState::ReadyToDeliver;
        std::mem::swap(&mut prev_delivery_state, &mut self.delivery_state);

        if let QueueSubscriberDeliveryState::OnDelivery(messages) = prev_delivery_state {
            return Some(messages);
        }

        return None;
    }

    pub fn set_messages_on_delivery(&mut self, messages_bucket: MessagesBucket) {
        let mut prev_delivery_state = QueueSubscriberDeliveryState::OnDelivery(messages_bucket);
        std::mem::swap(&mut prev_delivery_state, &mut self.delivery_state);

        if let QueueSubscriberDeliveryState::Rented = prev_delivery_state {
            return;
        }

        panic!(
            "We are setting messages on delivery but previous state is '{}'. Previous state must be 'Rented'",
            self.delivery_state.to_string()
        );
    }

    pub fn get_messages_amount_on_delivery(&self) -> usize {
        match &self.delivery_state {
            QueueSubscriberDeliveryState::ReadyToDeliver => 0,
            QueueSubscriberDeliveryState::Rented => 0,
            QueueSubscriberDeliveryState::OnDelivery(bucket) => bucket.messages_count(),
        }
    }
}
