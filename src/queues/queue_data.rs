use my_service_bus_shared::MessageId;

#[derive(Debug)]
pub struct NextMessage {
    pub message_id: MessageId,
    pub attempt_no: i32,
}

/*
pub struct QueueData {
    pub topic_id: String,
    pub queue_id: String,
}

impl QueueData {
    pub fn new(topic_id: String, queue_id: String, queue_type: TopicQueueType) -> Self {
        QueueData { topic_id, queue_id }
    }

    pub fn restore(
        topic_id: String,
        queue_id: String,
        queue_type: TopicQueueType,
        queue: QueueWithIntervals,
    ) -> Self {
        QueueData { topic_id, queue_id }
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
}

 */
