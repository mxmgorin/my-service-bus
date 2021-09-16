use my_service_bus_shared::{queue_with_intervals::QueueIndexRange, TopicQueueType};

pub struct TopicQueueSnapshot {
    pub queue_id: String,
    pub queue_type: TopicQueueType,
    pub ranges: Vec<QueueIndexRange>,
}

pub struct TopicSnapshot {
    pub topic_id: String,
    pub message_id: i64,
    pub queues: Vec<TopicQueueSnapshot>,
}
