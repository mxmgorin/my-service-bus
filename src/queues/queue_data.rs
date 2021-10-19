use std::collections::HashMap;

use my_service_bus_shared::{
    queue::TopicQueueType,
    queue_with_intervals::{QueueIndexRange, QueueWithIntervals},
    MessageId,
};
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    operations::OperationFailResult,
    queue_subscribers::{QueueSubscriber, SubscriberId, SubscribersList},
};

use super::TopicQueueMetrics;

#[derive(Debug)]
pub struct NextMessage {
    pub message_id: MessageId,
    pub attempt_no: i32,
}

pub struct QueueData {
    pub topic_id: String,
    pub queue_id: String,
    pub queue: QueueWithIntervals,
    pub subscribers: SubscribersList,
    attempts: HashMap<MessageId, i32>,
    pub queue_type: TopicQueueType,
    pub last_ubsubscribe: DateTimeAsMicroseconds,
}

impl QueueData {
    pub fn new(topic_id: String, queue_id: String, queue_type: TopicQueueType) -> Self {
        QueueData {
            topic_id,
            queue_id,
            queue: QueueWithIntervals::new(),
            subscribers: SubscribersList::new(queue_type),
            attempts: HashMap::new(),
            queue_type,
            last_ubsubscribe: DateTimeAsMicroseconds::now(),
        }
    }

    pub fn restore(
        topic_id: String,
        queue_id: String,
        queue_type: TopicQueueType,
        queue: QueueWithIntervals,
    ) -> Self {
        QueueData {
            topic_id,
            queue_id,
            queue,
            subscribers: SubscribersList::new(queue_type),
            attempts: HashMap::new(),
            queue_type,
            last_ubsubscribe: DateTimeAsMicroseconds::now(),
        }
    }

    pub fn queue_type_is_about_to_change(&self, new_queue_type: TopicQueueType) -> bool {
        match self.queue_type {
            TopicQueueType::Permanent => match new_queue_type {
                TopicQueueType::Permanent => false,
                TopicQueueType::DeleteOnDisconnect => true,
                TopicQueueType::PermanentWithSingleConnection => true,
            },
            TopicQueueType::DeleteOnDisconnect => match new_queue_type {
                TopicQueueType::Permanent => true,
                TopicQueueType::DeleteOnDisconnect => false,
                TopicQueueType::PermanentWithSingleConnection => true,
            },
            TopicQueueType::PermanentWithSingleConnection => match new_queue_type {
                TopicQueueType::Permanent => true,
                TopicQueueType::DeleteOnDisconnect => true,
                TopicQueueType::PermanentWithSingleConnection => false,
            },
        }
    }

    pub fn update_queue_type(&mut self, new_queue_type: TopicQueueType) {
        if !self.queue_type_is_about_to_change(new_queue_type) {
            return;
        }

        self.queue_type = new_queue_type;
        //T - cover all the cases of changing queue_type;
    }

    pub fn enqueue_messages(&mut self, msgs: &QueueWithIntervals) {
        for msg_id in msgs {
            self.queue.enqueue(msg_id);
        }
    }

    pub async fn update_metrics(&self, metrics: &TopicQueueMetrics) {
        metrics
            .update(self.queue.len(), self.queue.get_snapshot())
            .await;
    }

    pub fn get_attempt_no(&self, message_id: MessageId) -> i32 {
        match self.attempts.get(&message_id) {
            Some(result) => *result,
            None => 0,
        }
    }

    pub fn dequeue_next_message(&mut self) -> Option<NextMessage> {
        let message_id = self.queue.dequeue()?;

        let result = NextMessage {
            message_id,
            attempt_no: self.get_attempt_no(message_id),
        };

        Some(result)
    }

    pub fn peek_next_message(&mut self) -> Option<NextMessage> {
        let message_id = self.queue.peek()?;

        let result = NextMessage {
            message_id,
            attempt_no: self.get_attempt_no(message_id),
        };

        Some(result)
    }

    pub fn get_snapshot(&self) -> Vec<QueueIndexRange> {
        self.queue.get_snapshot()
    }

    pub fn confirmed_delivered(
        &mut self,
        subscriber_id: SubscriberId,
    ) -> Result<(), OperationFailResult> {
        let subscriber = self.subscribers.get_by_id_mut(subscriber_id);

        if subscriber.is_none() {
            return Err(OperationFailResult::SubscriberNotFound { id: subscriber_id });
        }

        let subscriber = subscriber.unwrap();

        let messages_bucket = subscriber.reset_delivery();

        if messages_bucket.is_none() {
            println!(
                "{}/{} confirmed_delivered: No messages on delivery at subscriber {}",
                self.topic_id, self.queue_id, subscriber_id
            );

            return Ok(());
        };

        let messages_bucket = messages_bucket.unwrap();

        update_delivery_time(
            subscriber,
            messages_bucket.messages_count_with_intermediary_confirmed(),
            true,
        );

        self.process_delivered(&messages_bucket.ids);

        Ok(())
    }

    pub fn confirmed_non_delivered(
        &mut self,
        subscriber_id: SubscriberId,
    ) -> Result<(), OperationFailResult> {
        let subscriber = self.subscribers.get_by_id_mut(subscriber_id);

        if subscriber.is_none() {
            return Err(OperationFailResult::SubscriberNotFound { id: subscriber_id });
        }

        let subscriber = subscriber.unwrap();

        let messages_bucket = subscriber.reset_delivery();

        if messages_bucket.is_none() {
            println!(
                "{}/{} confirmed_non_delivered: No messages on delivery at subscriber {}",
                self.topic_id, self.queue_id, subscriber_id
            );

            return Ok(());
        };

        let messages_bucket = messages_bucket.unwrap();

        update_delivery_time(
            subscriber,
            messages_bucket.messages_count_with_intermediary_confirmed(),
            false,
        );

        self.process_not_delivered(&messages_bucket.ids);

        Ok(())
    }

    pub fn intermediary_confirmed(
        &mut self,
        subscriber_id: SubscriberId,
        confirmed: QueueWithIntervals,
    ) -> Result<(), OperationFailResult> {
        let subscriber = self.subscribers.get_by_id_mut(subscriber_id);

        if subscriber.is_none() {
            return Err(OperationFailResult::SubscriberNotFound { id: subscriber_id });
        }

        let subscriber = subscriber.unwrap();

        if confirmed.len() > 0 {
            subscriber.intermediary_confirmed(&confirmed);
            self.process_delivered(&confirmed);
        }

        return Ok(());
    }

    pub fn confirmed_some_delivered(
        &mut self,
        subscriber_id: SubscriberId,
        delivered: QueueWithIntervals,
    ) -> Result<(), OperationFailResult> {
        let subscriber = self.subscribers.get_by_id_mut(subscriber_id);

        if subscriber.is_none() {
            return Err(OperationFailResult::SubscriberNotFound { id: subscriber_id });
        }

        let subscriber = subscriber.unwrap();

        let messages_bucket = subscriber.reset_delivery();

        if messages_bucket.is_none() {
            println!(
                "{}/{} confirmed_some_delivered: No messages on delivery at subscriber {}",
                self.topic_id, self.queue_id, subscriber_id
            );

            return Ok(());
        };

        let mut messages_bucket = messages_bucket.unwrap();

        //Remove delivered and what remains - is not delivered
        for delivered_message_id in &delivered {
            messages_bucket.remove(delivered_message_id);
        }

        update_delivery_time(
            subscriber,
            messages_bucket.messages_count_with_intermediary_confirmed(),
            false,
        );

        if messages_bucket.ids.len() > 0 {
            self.process_not_delivered(&messages_bucket.ids);
        } else {
            self.process_delivered(&delivered);
        }

        Ok(())
    }

    fn process_delivered(&mut self, delivered_ids: &QueueWithIntervals) {
        for msg_id in delivered_ids {
            self.attempts.remove(&msg_id);
        }
    }

    pub fn process_not_delivered(&mut self, not_delivered_ids: &QueueWithIntervals) {
        self.queue.merge_with(not_delivered_ids);

        for msg_id in not_delivered_ids {
            self.add_attempt(msg_id);
        }
    }

    fn add_attempt(&mut self, message_id: MessageId) {
        let result = self.attempts.get(&message_id);

        match result {
            Some(value) => {
                let value = value.clone();
                self.attempts.insert(message_id, value + 1);
            }
            None => {
                self.attempts.insert(message_id, 0);
            }
        }
    }
}

fn update_delivery_time(subscriber: &mut QueueSubscriber, amount: usize, positive: bool) {
    let delivery_duration =
        DateTimeAsMicroseconds::now().duration_since(subscriber.metrics.start_delivery_time);

    if positive {
        subscriber
            .metrics
            .set_delivered_statistic(amount, delivery_duration);
    } else {
        subscriber
            .metrics
            .set_not_delivered_statistic(amount as i32, delivery_duration);
    }
}
