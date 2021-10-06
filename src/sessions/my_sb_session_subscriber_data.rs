use my_service_bus_shared::date_time::DateTimeAsMicroseconds;

use crate::metric_data::{MetricOneSecond, MetricsHistory};

#[derive(Clone)]
pub struct MySbSessionSubscriberData {
    pub topic_id: String,
    pub queue_id: String,
    pub active: u8,
    pub subscribed: DateTimeAsMicroseconds,
    pub metrics: MetricsHistory,

    pub delivered_amount: MetricOneSecond,
    pub delivery_microseconds: MetricOneSecond,
}

impl MySbSessionSubscriberData {
    pub fn new(topic_id: &str, queue_id: &str, active: u8) -> Self {
        Self {
            topic_id: topic_id.to_string(),
            queue_id: queue_id.to_string(),
            active,
            subscribed: DateTimeAsMicroseconds::now(),
            metrics: MetricsHistory::new(),
            delivered_amount: MetricOneSecond::new(),
            delivery_microseconds: MetricOneSecond::new(),
        }
    }

    pub fn one_second_tick(&mut self) {
        if self.active > 0 {
            self.active -= 1;
        }

        let delivered_amount = self.delivered_amount.get_and_reset();
        let delivery_microseconds = self.delivery_microseconds.get_and_reset();

        if delivery_microseconds > 0 {
            let delivered = delivery_microseconds / delivered_amount;
            self.metrics.put(delivered as i32);
        }
    }

    pub fn to_string(&self) -> String {
        return format!(
            "TopicId: {}, QueueId: {}, subscribed_at: {}",
            self.topic_id,
            self.queue_id,
            self.subscribed.to_rfc3339()
        );
    }
}
