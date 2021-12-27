use std::{collections::HashMap, time::Duration};

use my_service_bus_shared::{queue::TopicQueueType, MessageId};
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::tcp::tcp_server::ConnectionId;

use super::{QueueSubscriber, SubscriberId};

pub enum SubscribersData {
    MultiSubscribers(HashMap<SubscriberId, QueueSubscriber>),
    SingleSubscriber(Option<QueueSubscriber>),
}

pub struct DeadSubscriber {
    pub subscriber_id: SubscriberId,
    pub session_id: i64,
    pub duration: Duration,
}

impl DeadSubscriber {
    pub fn new(subscriber: &QueueSubscriber, duration: Duration) -> Self {
        Self {
            session_id: subscriber.session_id,
            subscriber_id: subscriber.id,
            duration,
        }
    }
}

pub struct SubscribersList {
    data: SubscribersData,
    pub snapshot_id: usize,
    pub last_unsubscribe: DateTimeAsMicroseconds,
}

#[derive(Debug)]
pub enum SubscribeErrorResult {
    SubscriberWithIdExists,
    SubscriberOfSameConnectionExists,
}

impl SubscribersList {
    pub fn new(queue_type: TopicQueueType) -> Self {
        let last_unsubscribe = DateTimeAsMicroseconds::now();
        match queue_type {
            TopicQueueType::Permanent => Self {
                snapshot_id: 0,
                data: SubscribersData::MultiSubscribers(HashMap::new()),
                last_unsubscribe,
            },
            TopicQueueType::DeleteOnDisconnect => Self {
                snapshot_id: 0,
                data: SubscribersData::MultiSubscribers(HashMap::new()),
                last_unsubscribe,
            },
            TopicQueueType::PermanentWithSingleConnection => Self {
                snapshot_id: 0,
                data: SubscribersData::SingleSubscriber(None),
                last_unsubscribe,
            },
        }
    }

    pub fn get_all(&self) -> Option<Vec<&QueueSubscriber>> {
        match &self.data {
            SubscribersData::MultiSubscribers(hash_map) => {
                if hash_map.len() == 0 {
                    return None;
                }

                return Some(hash_map.values().collect());
            }
            SubscribersData::SingleSubscriber(single) => {
                let subscriber = single.as_ref()?;
                Some(vec![subscriber])
            }
        }
    }

    pub fn get_and_rent_next_subscriber_ready_to_deliver(
        &mut self,
    ) -> Option<&mut QueueSubscriber> {
        match &mut self.data {
            SubscribersData::MultiSubscribers(state) => {
                for subscriber in state.values_mut() {
                    if subscriber.rent_me() {
                        return Some(subscriber);
                    }
                }
            }
            SubscribersData::SingleSubscriber(state) => {
                if let Some(subscriber) = state {
                    if subscriber.rent_me() {
                        return Some(subscriber);
                    }
                }
            }
        }

        None
    }

    pub fn get_by_id(&self, subscriber_id: SubscriberId) -> Option<&QueueSubscriber> {
        match &self.data {
            SubscribersData::MultiSubscribers(hash_map) => return hash_map.get(&subscriber_id),
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

    fn check_that_we_has_already_subscriber_for_that_session(
        &self,
        connection_id: ConnectionId,
    ) -> Result<(), SubscribeErrorResult> {
        match &self.data {
            SubscribersData::MultiSubscribers(hash_map) => {
                for subscriber in hash_map.values() {
                    if subscriber.session_id == connection_id {
                        return Err(SubscribeErrorResult::SubscriberOfSameConnectionExists);
                    }
                }
            }
            SubscribersData::SingleSubscriber(single_subscriber) => {
                if let Some(subscriber) = single_subscriber {
                    if subscriber.session_id == connection_id {
                        return Err(SubscribeErrorResult::SubscriberOfSameConnectionExists);
                    }
                }
            }
        }

        Ok(())
    }

    ///Returns the subscriber which is kicked
    pub fn subscribe(
        &mut self,
        subscriber_id: SubscriberId,
        topic_id: String,
        queue_id: String,
        session_id: i64,
        delivery_packet_version: i32,
    ) -> Result<Option<QueueSubscriber>, SubscribeErrorResult> {
        self.check_that_we_has_already_subscriber_for_that_session(session_id)?;
        self.snapshot_id += 1;

        match &mut self.data {
            SubscribersData::MultiSubscribers(hash_map) => {
                if hash_map.contains_key(&subscriber_id) {
                    return Err(SubscribeErrorResult::SubscriberWithIdExists);
                }

                let subscriber = QueueSubscriber::new(
                    subscriber_id,
                    topic_id,
                    queue_id,
                    session_id,
                    delivery_packet_version,
                );

                hash_map.insert(subscriber_id, subscriber);

                return Ok(None);
            }
            SubscribersData::SingleSubscriber(single) => {
                if let Some(subscriber) = single {
                    if subscriber.id == subscriber_id {
                        return Err(SubscribeErrorResult::SubscriberWithIdExists);
                    }
                }

                let mut old_subscriber = Some(QueueSubscriber::new(
                    subscriber_id,
                    topic_id,
                    queue_id,
                    session_id,
                    delivery_packet_version,
                ));

                std::mem::swap(&mut old_subscriber, single);

                return Ok(old_subscriber);
            }
        }
    }

    pub fn get_amount(&self) -> usize {
        match &self.data {
            SubscribersData::MultiSubscribers(hash_map) => hash_map.len(),
            SubscribersData::SingleSubscriber(single) => {
                if single.is_none() {
                    0
                } else {
                    1
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

    fn resolve_subscriber_id_by_connection_id(
        &self,
        connection_id: ConnectionId,
    ) -> Option<SubscriberId> {
        match &self.data {
            SubscribersData::MultiSubscribers(hash_map) => {
                for sub in hash_map.values() {
                    if sub.session_id == connection_id {
                        return Some(sub.id);
                    }
                }
            }
            SubscribersData::SingleSubscriber(single) => {
                if let Some(sub) = single {
                    if sub.session_id == connection_id {
                        return Some(sub.id);
                    }
                }
            }
        }

        None
    }

    pub fn remove(&mut self, subscriber_id: SubscriberId) -> Option<QueueSubscriber> {
        match &mut self.data {
            SubscribersData::MultiSubscribers(multi) => {
                let result = multi.remove(&subscriber_id);
                if result.is_some() {
                    self.last_unsubscribe = DateTimeAsMicroseconds::now();
                }
                self.snapshot_id += 1;
                result
            }
            SubscribersData::SingleSubscriber(single) => {
                let mut result = None;

                if let Some(sub) = single {
                    if sub.id == subscriber_id {
                        self.last_unsubscribe = DateTimeAsMicroseconds::now();
                        std::mem::swap(&mut result, single);
                    }
                }
                self.snapshot_id += 1;
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

    pub fn find_subscribers_dead_on_delivery(
        &self,
        max_delivery_duration: Duration,
    ) -> Option<Vec<DeadSubscriber>> {
        match &self.data {
            SubscribersData::MultiSubscribers(subscribers) => {
                let mut result = None;

                for subscriber in subscribers.values() {
                    if let Some(duration) = subscriber.is_dead_on_delivery(max_delivery_duration) {
                        if result.is_none() {
                            result = Some(Vec::new());
                        }

                        if let Some(result) = &mut result {
                            result.push(DeadSubscriber::new(subscriber, duration));
                        }
                    }
                }

                return result;
            }
            SubscribersData::SingleSubscriber(state) => match state {
                Some(subscriber) => {
                    if let Some(duration) = subscriber.is_dead_on_delivery(max_delivery_duration) {
                        return Some(vec![DeadSubscriber::new(subscriber, duration)]);
                    }

                    return None;
                }
                None => return None,
            },
        }
    }

    pub fn get_min_message_id(&self) -> Option<MessageId> {
        match &self.data {
            SubscribersData::MultiSubscribers(hash_map) => {
                let mut result = None;
                for subscriber in hash_map.values() {
                    let subscriber_min_id = subscriber.get_min_message_id();

                    if let Some(result_min_id) = result {
                        if let Some(subscriber_min_id) = subscriber_min_id {
                            if subscriber_min_id < result_min_id {
                                result = Some(subscriber_min_id)
                            }
                        }
                    } else {
                        result = subscriber_min_id;
                    }
                }

                return result;
            }
            SubscribersData::SingleSubscriber(single_subscriber) => {
                if let Some(single_subscriber) = single_subscriber {
                    return single_subscriber.get_min_message_id();
                }
                return None;
            }
        }
    }
}
