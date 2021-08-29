use tokio::sync::RwLock;

use crate::metric_data::{MetricOneSecond, MetricsHistory};

pub struct TopicMetricsReadData {
    pub messages_per_second: usize,
    pub packets_per_second: usize,
    pub persist_queue_size: i64,
    pub publish_history: Vec<i32>,
}

pub struct TopicMetricsData {
    messages_per_second_going: MetricOneSecond,
    packets_per_second_going: MetricOneSecond,

    pub persist_queue_size: i64,

    pub messages_per_second: usize,
    pub packets_per_second: usize,

    pub publish_history: MetricsHistory,
}

impl TopicMetricsData {
    pub fn new() -> Self {
        Self {
            messages_per_second_going: MetricOneSecond::new(),
            packets_per_second_going: MetricOneSecond::new(),
            messages_per_second: 0,
            packets_per_second: 0,
            publish_history: MetricsHistory::new(),
            persist_queue_size: 0,
        }
    }
}

pub struct TopicMetrics {
    data: RwLock<TopicMetricsData>,
}

impl TopicMetrics {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(TopicMetricsData::new()),
        }
    }

    pub async fn update_topic_metrics(&self, new_messages_count: usize) {
        let mut write_access = self.data.write().await;

        write_access
            .messages_per_second_going
            .increase(new_messages_count);

        write_access.packets_per_second_going.increase(1);
    }

    pub async fn one_second_tick(&self, persist_queue_size: i64) {
        let mut write_access = self.data.write().await;

        let messages_per_second = write_access.messages_per_second_going.get_and_reset();
        write_access.packets_per_second = write_access.packets_per_second_going.get_and_reset();
        write_access.messages_per_second = messages_per_second;
        write_access.persist_queue_size = persist_queue_size;

        write_access.publish_history.put(messages_per_second as i32);
    }

    pub async fn get(&self) -> TopicMetricsReadData {
        let read_access = self.data.read().await;

        TopicMetricsReadData {
            packets_per_second: read_access.packets_per_second,
            messages_per_second: read_access.messages_per_second,
            publish_history: read_access.publish_history.get(),
            persist_queue_size: read_access.persist_queue_size,
        }
    }
}
