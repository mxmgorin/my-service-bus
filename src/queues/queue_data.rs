use std::collections::HashMap;

use my_service_bus_shared::{
    date_time::DateTimeAsMicroseconds,
    queue::TopicQueueType,
    queue_with_intervals::{QueueIndexRange, QueueWithIntervals},
    MessageId,
};

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
        update_delivery_time(subscriber, true);

        let messages_bucket = subscriber
            .reset_delivery()
            .expect(format!("No messages on delivery at subscriber {}", subscriber_id).as_str());

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
        update_delivery_time(subscriber, false);

        let messages_bucket = subscriber
            .reset_delivery()
            .expect(format!("No messages on delivery at subscriber {}", subscriber_id).as_str());

        self.process_not_delivered(&messages_bucket.ids);

        Ok(())
    }

    pub fn confirmed_some_not_delivered(
        &mut self,
        subscriber_id: SubscriberId,
        not_delivered: QueueWithIntervals,
    ) -> Result<(), OperationFailResult> {
        let subscriber = self.subscribers.get_by_id_mut(subscriber_id);

        if subscriber.is_none() {
            return Err(OperationFailResult::SubscriberNotFound { id: subscriber_id });
        }

        let subscriber = subscriber.unwrap();
        update_delivery_time(subscriber, false);

        let mut messages_bucket = subscriber
            .reset_delivery()
            .expect(format!("No messages on delivery at subscriber {}", subscriber_id).as_str());

        //We are removing all not delivered and what remains - is what was delivered
        for not_delivered_message_id in &not_delivered {
            messages_bucket.remove(not_delivered_message_id);
        }

        if messages_bucket.messages_count() > 0 {
            self.process_delivered(&messages_bucket.ids);
        }

        self.process_not_delivered(&not_delivered);

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
        update_delivery_time(subscriber, false);

        let mut messages_bucket = subscriber
            .reset_delivery()
            .expect(format!("No messages on delivery at subscriber {}", subscriber_id).as_str());

        //Remove delivered and what remains - is not delivered
        for delivered_message_id in &delivered {
            messages_bucket.remove(delivered_message_id);
        }

        self.process_delivered(&delivered);

        if messages_bucket.ids.len() > 0 {
            self.process_not_delivered(&messages_bucket.ids);
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

fn update_delivery_time(subscriber: &mut QueueSubscriber, positive: bool) {
    let messages_on_delivery = subscriber.get_messages_amount_on_delivery();

    let delivery_duration =
        DateTimeAsMicroseconds::now().duration_since(subscriber.metrics.start_delivery_time);

    if positive {
        subscriber
            .metrics
            .set_delivered_statistic(messages_on_delivery, delivery_duration);
    } else {
        subscriber
            .metrics
            .set_not_delivered_statistic(messages_on_delivery as i32, delivery_duration);
    }
}
