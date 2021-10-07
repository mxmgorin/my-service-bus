use std::{collections::HashMap, sync::Arc};

use my_service_bus_shared::queue::TopicQueueType;

use crate::{
    messages_bucket::MessagesBucket, queues::TopicQueue, sessions::MyServiceBusSession,
    tcp::tcp_server::ConnectionId, topics::Topic,
};

use super::{QueueSubscriber, SubscriberId, SubscriberMetrics};

pub enum SubscribersData {
    MultiSubscribers(HashMap<SubscriberId, QueueSubscriber>),
    SingleSubscriber(Option<QueueSubscriber>),
}

pub struct SubscribersList {
    data: SubscribersData,
}

impl SubscribersList {
    pub fn new(queue_type: TopicQueueType) -> Self {
        match queue_type {
            TopicQueueType::Permanent => Self {
                data: SubscribersData::MultiSubscribers(HashMap::new()),
            },
            TopicQueueType::DeleteOnDisconnect => Self {
                data: SubscribersData::MultiSubscribers(HashMap::new()),
            },
            TopicQueueType::PermanentWithSingleConnection => Self {
                data: SubscribersData::SingleSubscriber(None),
            },
        }
    }

    pub fn get_and_rent_next_subscriber_ready_to_deliver(&mut self) -> Option<&QueueSubscriber> {
        match &mut self.data {
            SubscribersData::MultiSubscribers(state) => {
                for subscriber in state.values_mut() {
                    if !subscriber.rented {
                        subscriber.rented = true;
                        return Some(subscriber);
                    }
                }
            }
            SubscribersData::SingleSubscriber(state) => {
                if let Some(state) = state {
                    if !state.rented {
                        state.rented = true;
                        return Some(state);
                    }
                }
            }
        }

        None
    }

    pub fn get_by_id_mut(&mut self, subscriber_id: SubscriberId) -> Option<&mut QueueSubscriber> {
        match &mut self.data {
            SubscribersData::MultiSubscribers(hash_map) => return hash_map.get_mut(&subscriber_id),
            SubscribersData::SingleSubscriber(single) => {
                if let Some(subscriber) = single {
                    if subscriber.id == subscriber_id {
                        return Some(subscriber);
                    }
                }

                return None;
            }
        }
    }

    pub fn set_messages_on_delivery(
        &mut self,
        subscriber_id: SubscriberId,
        messages: MessagesBucket,
    ) {
        let item = self.get_by_id_mut(subscriber_id);

        if let Some(subscriber) = item {
            subscriber.messages_on_delivery = Some(messages);
            subscriber.metrics.set_started_delivery();
        } else {
            panic!(
                "Can not set messages on delivery . Subscriber {} is not found",
                subscriber_id
            )
        }
    }

    pub fn subscribe(
        &mut self,
        subscriber_id: SubscriberId,
        connection_id: ConnectionId,
        topic: Arc<Topic>,
        queue: Arc<TopicQueue>,
        session: Arc<MyServiceBusSession>,
    ) -> Option<QueueSubscriber> {
        match &mut self.data {
            SubscribersData::MultiSubscribers(hash_map) => {
                if hash_map.contains_key(&subscriber_id) {
                    panic!("Can not add subscriber with {}. Subscriber with the same ID is already in the multilist", subscriber_id);
                }

                let subscriber =
                    QueueSubscriber::new(subscriber_id, connection_id, topic, queue, session);

                hash_map.insert(subscriber_id, subscriber);

                return None;
            }
            SubscribersData::SingleSubscriber(single) => {
                if let Some(subscriber) = single {
                    if subscriber.id == subscriber_id {
                        panic!("Can not add subscriber with {}. Subscriber with the same ID is already in the singlelist", subscriber_id);
                    }
                }

                let mut old_subscriber = Some(QueueSubscriber::new(
                    subscriber_id,
                    connection_id,
                    topic,
                    queue,
                    session,
                ));

                std::mem::swap(&mut old_subscriber, single);

                return old_subscriber;
            }
        }
    }

    pub fn reset_rented(&mut self, subscriber_id: SubscriberId) {
        let found_subscriber = self.get_by_id_mut(subscriber_id);

        if let Some(subscriber) = found_subscriber {
            subscriber.rented = false;
        }
    }

    pub fn get_amount(&self) -> usize {
        match &self.data {
            SubscribersData::MultiSubscribers(hash_map) => hash_map.len(),
            SubscribersData::SingleSubscriber(single) => {
                if single.is_none() {
                    1
                } else {
                    0
                }
            }
        }
    }

    pub fn one_second_tick(&mut self) {
        match &mut self.data {
            SubscribersData::MultiSubscribers(hash_map) => {
                for queue_subscriber in hash_map.values_mut() {
                    queue_subscriber.metrics.one_second_tick()
                }
            }
            SubscribersData::SingleSubscriber(single) => {
                if let Some(subscriber) = single {
                    subscriber.metrics.one_second_tick();
                }
            }
        }
    }

    pub fn get_all_subscriber_metrics(&self) -> Vec<SubscriberMetrics> {
        let mut result = Vec::new();

        match &self.data {
            SubscribersData::MultiSubscribers(hash_map) => {
                for sub in hash_map.values() {
                    result.push(sub.metrics.clone())
                }
            }
            SubscribersData::SingleSubscriber(single) => {
                if let Some(sub) = single {
                    result.push(sub.metrics.clone())
                }
            }
        }

        result
    }

    fn resolve_subscriber_id_by_connection_id(
        &self,
        connection_id: ConnectionId,
    ) -> Option<SubscriberId> {
        match &self.data {
            SubscribersData::MultiSubscribers(hash_map) => {
                for sub in hash_map.values() {
                    if sub.metrics.connection_id == connection_id {
                        return Some(sub.id);
                    }
                }
            }
            SubscribersData::SingleSubscriber(single) => {
                if let Some(sub) = single {
                    return Some(sub.id);
                }
            }
        }

        None
    }

    pub fn remove(&mut self, subscriber_id: SubscriberId) -> Option<QueueSubscriber> {
        match &mut self.data {
            SubscribersData::MultiSubscribers(multi) => multi.remove(&subscriber_id),
            SubscribersData::SingleSubscriber(single) => {
                let mut result = None;

                if let Some(sub) = single {
                    if sub.id == subscriber_id {
                        std::mem::swap(&mut result, single);
                    }
                }

                result
            }
        }
    }

    pub fn remove_by_connection_id(
        &mut self,
        connection_id: ConnectionId,
    ) -> Option<QueueSubscriber> {
        let subscriber_id = self.resolve_subscriber_id_by_connection_id(connection_id)?;
        self.remove(subscriber_id)
    }
}
