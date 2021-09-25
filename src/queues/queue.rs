use my_service_bus_shared::{
    queue::TopicQueueType,
    queue_with_intervals::{QueueIndexRange, QueueWithIntervals},
    MessageId,
};
use tokio::sync::RwLock;

use crate::date_time::MyDateTime;
use crate::topics::TopicQueueSnapshot;

use super::QueueData;

pub struct TopicQueueGcData {
    pub subscribers_amount: usize,
    pub queue_type: TopicQueueType,
    pub last_subscriber_disconnect: MyDateTime,
}

pub struct TopicQueueMonitoringData {
    pub id: String,
    pub queue_type: TopicQueueType,
    pub size: i64,
    pub queue: Vec<QueueIndexRange>,
}

pub struct TopicQueue {
    pub topic_id: String,
    pub queue_id: String,
    pub data: RwLock<QueueData>,
}

impl TopicQueue {
    pub fn new(topic_id: &str, queue_id: &str, queue_type: TopicQueueType) -> TopicQueue {
        let data = RwLock::new(QueueData::new(
            topic_id.to_string(),
            queue_id.to_string(),
            queue_type,
        ));

        let result = TopicQueue {
            topic_id: topic_id.to_string(),
            queue_id: queue_id.to_string(),
            data,
        };

        return result;
    }

    pub fn restore(
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

        let result = TopicQueue {
            topic_id: topic_id.to_string(),
            queue_id: queue_id.to_string(),
            data: RwLock::new(data),
        };

        return result;
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
}
