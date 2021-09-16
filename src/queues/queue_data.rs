use std::collections::HashMap;

use my_service_bus_shared::{
    queue_with_intervals::{QueueIndexRange, QueueWithIntervals},
    MessageId, TopicQueueType,
};

use crate::{
    date_time::MyDateTime, messages_bucket::MessagesBucket, operations::OperationFailResult,
    subscribers::SubscribersList,
};

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
    pub last_ubsubscribe: MyDateTime,
}

impl QueueData {
    pub fn new(topic_id: String, queue_id: String, queue_type: TopicQueueType) -> Self {
        QueueData {
            topic_id,
            queue_id,
            queue: QueueWithIntervals::new(),
            subscribers: SubscribersList::new(),
            attempts: HashMap::new(),
            queue_type,
            last_ubsubscribe: MyDateTime::utc_now(),
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
            subscribers: SubscribersList::new(),
            attempts: HashMap::new(),
            queue_type,
            last_ubsubscribe: MyDateTime::utc_now(),
        }
    }

    pub fn enqueue_messages(&mut self, msgs: &[MessageId]) {
        for msg_id in msgs {
            self.queue.enqueue(*msg_id);
        }
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

    pub fn confirmed_delivered(&mut self, messages: MessagesBucket) {
        for page in &messages.pages {
            for msg_id in page.messages.keys() {
                self.attempts.remove(msg_id);
            }
        }
    }

    pub fn confirmed_non_delivered(&mut self, messages: &MessagesBucket) {
        for page in &messages.pages {
            for msg_id in page.messages.keys() {
                self.queue.enqueue(*msg_id);
                self.add_attempt(*msg_id);
            }
        }
    }

    pub fn confirmed_some_not_delivered(
        &mut self,
        mut messages: MessagesBucket,
        not_delivered: QueueWithIntervals,
    ) -> Result<(), OperationFailResult> {
        for by_page_id in not_delivered.split_by_page_id() {
            if !messages.find_page(by_page_id.page_id) {
                let reason = format!(
                    "confirmed_some_not_delivered: There is a message in the page {}. But page is not found",
                    by_page_id.page_id
                );

                return Err(OperationFailResult::Other(reason));
            }

            for message_id in by_page_id.ids {
                if !messages.remove_message(by_page_id.page_id, message_id) {
                    let reason = format!(
                        "confirmed_some_not_delivered: There is a message as confimred not delivered {}. But it's not found",
                        message_id
                    );

                    return Err(OperationFailResult::Other(reason));
                }

                self.queue.enqueue(message_id);
                self.add_attempt(message_id);
            }
        }

        for page in messages.pages {
            for message_id in page.ids {
                self.attempts.remove(&message_id);
            }
        }

        return Ok(());
    }

    pub fn confirmed_some_delivered(
        &mut self,
        mut messages: MessagesBucket,
        delivered: QueueWithIntervals,
    ) -> Result<(), OperationFailResult> {
        for by_page_id in delivered.split_by_page_id() {
            if !messages.find_page(by_page_id.page_id) {
                let reason = format!(
                    "confirmed_some_delivered: There is a message in the page {}. But page is not found",
                    by_page_id.page_id
                );

                return Err(OperationFailResult::Other(reason));
            }

            for message_id in by_page_id.ids {
                if !messages.remove_message(by_page_id.page_id, message_id) {
                    let reason = format!(
                        "confirmed_some_delivered: There is a message as confimred not delivered {}. But it's not found",
                        message_id
                    );

                    return Err(OperationFailResult::Other(reason));
                }

                self.attempts.remove(&message_id);
            }
        }

        self.confirmed_non_delivered(&messages);

        Ok(())
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
