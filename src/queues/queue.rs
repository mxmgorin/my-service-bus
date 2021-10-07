use my_service_bus_shared::{
    date_time::DateTimeAsMicroseconds, queue::TopicQueueType,
    queue_with_intervals::QueueWithIntervals, MessageId,
};
use tokio::sync::{Mutex, RwLock};

use crate::topics::TopicQueueSnapshot;

use super::{QueueData, TopicQueueMetrics};

pub struct TopicQueueGcData {
    pub subscribers_amount: usize,
    pub queue_type: TopicQueueType,
    pub last_subscriber_disconnect: DateTimeAsMicroseconds,
}

pub struct TopicQueue {
    pub topic_id: String,
    pub queue_id: String,
    pub data: RwLock<QueueData>,
    pub delivery_lock: Mutex<usize>,
    pub metrics: TopicQueueMetrics,
}

impl TopicQueue {
    pub async fn new(topic_id: &str, queue_id: &str, queue_type: TopicQueueType) -> TopicQueue {
        let data = QueueData::new(topic_id.to_string(), queue_id.to_string(), queue_type);

        let metrics = TopicQueueMetrics::new(queue_id.to_string(), queue_type);
        data.update_metrics(&metrics).await;

        TopicQueue {
            topic_id: topic_id.to_string(),
            queue_id: queue_id.to_string(),
            data: RwLock::new(data),
            delivery_lock: Mutex::new(0),
            metrics: TopicQueueMetrics::new(queue_id.to_string(), queue_type),
        }
    }

    pub async fn restore(
        topic_id: &str,
        queue_id: &str,
        queue_type: TopicQueueType,
        queue: QueueWithIntervals,
    ) -> TopicQueue {
        let data = QueueData::restore(
            topic_id.to_string(),
            queue_id.to_string(),
            queue_type,
            queue,
        );

        let metrics = TopicQueueMetrics::new(queue_id.to_string(), queue_type);
        data.update_metrics(&metrics).await;

        TopicQueue {
            topic_id: topic_id.to_string(),
            queue_id: queue_id.to_string(),
            data: RwLock::new(data),
            delivery_lock: Mutex::new(0),
            metrics,
        }
    }

    pub async fn get_min_msg_id(&self) -> Option<MessageId> {
        let read_access = self.data.read().await;
        read_access.queue.get_min_id()
    }

    pub async fn get_snapshot(&self) -> TopicQueueSnapshot {
        let read = self.data.read().await;
        let queue_id = read.queue_id.to_string();

        TopicQueueSnapshot {
            queue_id,
            queue_type: read.queue_type.clone(),
            ranges: read.get_snapshot(),
        }
    }

    pub async fn get_gc_data(&self) -> TopicQueueGcData {
        let read_access = self.data.read().await;

        TopicQueueGcData {
            queue_type: read_access.queue_type,
            subscribers_amount: read_access.subscribers.get_amount(),
            last_subscriber_disconnect: read_access.last_ubsubscribe,
        }
    }

    pub async fn update_queue_type(&self, queue_type: TopicQueueType) {
        let mut write_access = self.data.write().await;
        write_access.queue_type = queue_type;
    }

    pub async fn get_queue_size(&self) -> i64 {
        let read_access = self.data.read().await;
        return read_access.queue.len();
    }

    pub async fn enqueue_messages(&self, msgs: &QueueWithIntervals) {
        let mut write_access = self.data.write().await;
        write_access.enqueue_messages(msgs);
        write_access.update_metrics(&self.metrics).await;
    }
}
