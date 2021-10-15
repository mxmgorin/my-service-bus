use std::time::Duration;

use my_service_bus_shared::{
    date_time::DateTimeAsMicroseconds,
    messages_bucket::MessagesBucket,
    queue::TopicQueueType,
    queue_with_intervals::{QueueIndexRange, QueueWithIntervals},
    MessageId,
};
use tokio::sync::{Mutex, RwLock};

use crate::{
    app::AppContext,
    operations::OperationFailResult,
    queue_subscribers::{DeadSubscriber, QueueSubscriber, SubscriberId, SubscriberMetrics},
    tcp::tcp_server::ConnectionId,
    topics::TopicQueueSnapshot,
    utils::rw_locks::RwWriteAccess,
};

use super::{QueueData, TopicQueueMetrics};

pub struct TopicQueueGcData {
    pub subscribers_amount: usize,
    pub queue_type: TopicQueueType,
    pub last_subscriber_disconnect: DateTimeAsMicroseconds,
}

pub struct TopicQueue {
    pub topic_id: String,
    pub queue_id: String,
    data: RwLock<QueueData>,
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

    pub async fn get_snapshot_to_persist(&self) -> Option<TopicQueueSnapshot> {
        let read = self.data.read().await;

        match read.queue_type {
            TopicQueueType::Permanent => {
                let result = TopicQueueSnapshot {
                    queue_id: read.queue_id.to_string(),
                    queue_type: read.queue_type.clone(),
                    ranges: read.get_snapshot(),
                };

                Some(result)
            }
            TopicQueueType::DeleteOnDisconnect => None,
            TopicQueueType::PermanentWithSingleConnection => {
                let result = TopicQueueSnapshot {
                    queue_id: read.queue_id.to_string(),
                    queue_type: read.queue_type.clone(),
                    ranges: read.get_snapshot(),
                };

                Some(result)
            }
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

    pub async fn one_second_tick(&self) {
        let mut write_access = self.data.write().await;

        write_access.subscribers.one_second_tick();
    }

    pub async fn get_write_access<'a>(
        &'a self,
        process_id: i64,
        process: String,
        app: &AppContext,
    ) -> RwWriteAccess<'a, QueueData> {
        let write_access = self.data.write().await;
        return RwWriteAccess::new(write_access, process_id, process, app.locks.clone());
    }

    pub async fn get_all_subscribers_metrics(&self) -> Vec<SubscriberMetrics> {
        let mut result = Vec::new();

        let read_acess = self.data.read().await;

        let metrics_vec = read_acess.subscribers.get_all_subscriber_metrics();

        result.extend(metrics_vec);

        result
    }

    pub async fn find_subscribers_dead_on_delivery(
        &self,
        max_delivery_duration: Duration,
    ) -> Option<Vec<DeadSubscriber>> {
        let mut result = Vec::new();
        let write_access = self.data.write().await;

        let dead_on_delivery = write_access
            .subscribers
            .find_subscribers_dead_on_delivery(max_delivery_duration);

        if let Some(dead_on_delivery) = dead_on_delivery {
            result.extend(dead_on_delivery);
        }

        if result.len() > 0 {
            return Some(result);
        }

        return None;
    }

    #[inline]
    pub async fn remove_subscribers_by_connection_id(
        &self,
        connection_id: ConnectionId,
    ) -> Option<QueueSubscriber> {
        let mut write_access = self.data.write().await;

        let result = write_access
            .subscribers
            .remove_by_connection_id(connection_id);

        if result.is_some() {
            write_access.last_ubsubscribe = DateTimeAsMicroseconds::now();
        }

        result
    }

    pub async fn set_message_id(&self, message_id: MessageId, max_message_id: MessageId) {
        let mut topic_queue_data = self.data.write().await;

        let mut intervals = Vec::new();

        intervals.push(QueueIndexRange {
            from_id: message_id,
            to_id: max_message_id,
        });

        topic_queue_data.queue.reset(intervals);
    }

    pub async fn mark_not_delivered(&self, messages_on_delivery: &MessagesBucket) {
        let mut write_access = self.data.write().await;
        write_access.process_not_delivered(&messages_on_delivery.ids);
    }

    pub async fn confirmed_delivered(
        &self,
        subscriber_id: SubscriberId,
    ) -> Result<(), OperationFailResult> {
        let mut write_access = self.data.write().await;
        write_access.confirmed_delivered(subscriber_id)
    }

    pub async fn confirmed_non_delivered(
        &self,
        subscriber_id: SubscriberId,
    ) -> Result<(), OperationFailResult> {
        let mut write_access = self.data.write().await;
        write_access.confirmed_non_delivered(subscriber_id)
    }

    pub async fn confirmed_some_delivered(
        &self,
        subscriber_id: SubscriberId,
        delivered: QueueWithIntervals,
    ) -> Result<(), OperationFailResult> {
        let mut write_access = self.data.write().await;
        write_access.confirmed_some_delivered(subscriber_id, delivered)
    }

    pub async fn confirmed_some_not_delivered(
        &self,
        subscriber_id: SubscriberId,
        not_delivered: QueueWithIntervals,
    ) -> Result<(), OperationFailResult> {
        let mut write_access = self.data.write().await;
        write_access.confirmed_some_not_delivered(subscriber_id, not_delivered)
    }
}
