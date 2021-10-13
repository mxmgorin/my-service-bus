use std::{sync::Arc, time::Duration};

use my_service_bus_shared::{date_time::DateTimeAsMicroseconds, messages_bucket::MessagesBucket};

use crate::{queues::TopicQueue, sessions::MyServiceBusSession, topics::Topic};

use super::{SubscriberId, SubscriberMetrics};

pub struct OnDeliveryStateData {
    messages: MessagesBucket,
    inserted: DateTimeAsMicroseconds,
}
pub enum QueueSubscriberDeliveryState {
    ReadyToDeliver,
    Rented,
    OnDelivery(OnDeliveryStateData),
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
        topic: Arc<Topic>,
        queue: Arc<TopicQueue>,
        session: Arc<MyServiceBusSession>,
    ) -> Self {
        Self {
            topic: topic.clone(),
            queue: queue.clone(),
            subscribed: DateTimeAsMicroseconds::now(),
            metrics: SubscriberMetrics::new(id, session.id, topic, queue),
            delivery_state: QueueSubscriberDeliveryState::ReadyToDeliver,
            session,
            id,
        }
    }

    pub fn rent_me(&mut self) -> bool {
        if let QueueSubscriberDeliveryState::ReadyToDeliver = &self.delivery_state {
            self.metrics.set_delivery_mode_as_rented();
            self.delivery_state = QueueSubscriberDeliveryState::Rented;
            return true;
        }

        return false;
    }

    pub fn cancel_the_rent(&mut self) {
        if let QueueSubscriberDeliveryState::Rented = &self.delivery_state {
            self.metrics.set_delivery_mode_as_ready_to_deliver();
            self.delivery_state = QueueSubscriberDeliveryState::ReadyToDeliver;
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

        self.metrics.set_delivery_mode_as_ready_to_deliver();
        if let QueueSubscriberDeliveryState::OnDelivery(state) = prev_delivery_state {
            return Some(state.messages);
        }

        return None;
    }

    pub fn set_messages_on_delivery(&mut self, messages_bucket: MessagesBucket) {
        if let QueueSubscriberDeliveryState::Rented = &self.delivery_state {
            self.delivery_state = QueueSubscriberDeliveryState::OnDelivery(OnDeliveryStateData {
                messages: messages_bucket,
                inserted: DateTimeAsMicroseconds::now(),
            });
            self.metrics.set_delivery_mode_as_on_delivery();
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
            QueueSubscriberDeliveryState::OnDelivery(state) => state.messages.messages_count(),
        }
    }

    pub fn is_dead_on_delivery(&self, max_delivery_duration: Duration) -> Option<Duration> {
        match &self.delivery_state {
            QueueSubscriberDeliveryState::ReadyToDeliver => None,
            QueueSubscriberDeliveryState::Rented => None,
            QueueSubscriberDeliveryState::OnDelivery(state) => {
                let now = DateTimeAsMicroseconds::now();
                let duration = now.duration_since(state.inserted);
                if duration > max_delivery_duration {
                    return Some(duration);
                }

                return None;
            }
        }
    }
}
