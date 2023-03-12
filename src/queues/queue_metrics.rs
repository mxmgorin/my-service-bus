use my_service_bus_abstractions::queue_with_intervals::QueueIndexRange;
use my_service_bus_abstractions::subscriber::TopicQueueType;
use my_service_bus_shared::{queue_with_intervals::QueueIndexRange};

#[derive(Clone)]
pub struct TopicQueueMetrics {
    pub id: String,
    pub queue_type: TopicQueueType,
    pub size: i64,
    pub queue: Vec<QueueIndexRange>,
}
