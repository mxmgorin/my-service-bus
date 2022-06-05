use crate::{
    messages_page::PageSizeMetrics,
    metric_data::{MetricOneSecond, MetricsHistory},
};

pub struct TopicMetrics {
    messages_per_second_going: MetricOneSecond,
    packets_per_second_going: MetricOneSecond,

    pub persist_queue_size: usize,

    pub messages_per_second: usize,
    pub packets_per_second: usize,

    pub publish_history: MetricsHistory,

    pub size_metrics: PageSizeMetrics,
}

impl TopicMetrics {
    pub fn new() -> Self {
        Self {
            messages_per_second_going: MetricOneSecond::new(),
            packets_per_second_going: MetricOneSecond::new(),
            messages_per_second: 0,
            packets_per_second: 0,
            publish_history: MetricsHistory::new(),
            persist_queue_size: 0,
            size_metrics: PageSizeMetrics::new(),
        }
    }

    pub fn update_size_metrics(&mut self, metrics: &PageSizeMetrics) {
        self.size_metrics.update(metrics);
    }

    pub fn update_topic_metrics(&mut self, new_messages_count: usize) {
        self.messages_per_second_going.increase(new_messages_count);

        self.packets_per_second_going.increase(1);
    }

    pub fn one_second_tick(&mut self, persist_queue_size: usize) {
        let messages_per_second = self.messages_per_second_going.get_and_reset();
        self.packets_per_second = self.packets_per_second_going.get_and_reset();
        self.messages_per_second = messages_per_second;
        self.persist_queue_size = persist_queue_size;

        self.publish_history.put(messages_per_second as i32);
    }
}
