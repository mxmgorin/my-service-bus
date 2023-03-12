use my_service_bus_abstractions::queue_with_intervals::QueueIndexRange;
use my_service_bus_abstractions::subscriber::TopicQueueType;
use my_service_bus_shared::{queue_with_intervals::QueueIndexRange};

#[derive(Clone)]
pub struct TopicQueueSnapshot {
    pub queue_id: String,
    pub queue_type: TopicQueueType,
    pub ranges: Vec<QueueIndexRange>,
}
#[derive(Clone)]
pub struct TopicSnapshot {
    pub topic_id: String,
    pub message_id: i64,
    pub queues: Vec<TopicQueueSnapshot>,
}
