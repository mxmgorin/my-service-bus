use my_service_bus_shared::{queue::TopicQueueType, queue_with_intervals::QueueIndexRange};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct TopicQueueMetricsData {
    pub id: String,
    pub queue_type: TopicQueueType,
    pub size: i64,
    pub queue: Vec<QueueIndexRange>,
}

pub struct TopicQueueMetrics {
    data: Mutex<TopicQueueMetricsData>,
}

impl TopicQueueMetrics {
    pub fn new(queue_id: String, queue_type: TopicQueueType) -> Self {
        Self {
            data: Mutex::new(TopicQueueMetricsData {
                id: queue_id,
                queue_type,
                size: 0,
                queue: Vec::new(),
            }),
        }
    }

    pub async fn update(&self, size: i64, queue: Vec<QueueIndexRange>) {
        let mut write_acces = self.data.lock().await;
        write_acces.size = size;
        write_acces.queue = queue;
    }

    pub async fn get(&self) -> TopicQueueMetricsData {
        let read_access = self.data.lock().await;
        read_access.clone()
    }
}
