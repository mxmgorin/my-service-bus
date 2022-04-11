use prometheus::{Encoder, IntGaugeVec, Opts, Registry, TextEncoder};

pub struct PrometheusMetrics {
    registry: Registry,
    pub persist_queue_size: IntGaugeVec,
    pub topic_queue_size: IntGaugeVec,
}

impl PrometheusMetrics {
    pub fn new() -> Self {
        let registry = Registry::new();

        let gauge_opts = Opts::new("topic_persist_queue_size", "Topic queue to persist size");

        let lables = &["topic"];

        // This unwraps runs on application start. If it fails - applications is not started
        let persist_queue_size = IntGaugeVec::new(gauge_opts, lables).unwrap();

        registry
            .register(Box::new(persist_queue_size.clone()))
            .unwrap();

        let gauge_opts = Opts::new("topic_queue_size", "Topic queue size");

        let lables = &["topic", "queue"];

        // This unwraps runs on application start. If it fails - applications is not started
        let topic_queue_size = IntGaugeVec::new(gauge_opts, lables).unwrap();

        registry
            .register(Box::new(topic_queue_size.clone()))
            .unwrap();

        return Self {
            registry,
            persist_queue_size,
            topic_queue_size,
        };
    }

    pub fn update_persist_queue_size(&self, topic_id: &str, value: i64) {
        self.persist_queue_size
            .with_label_values(&[topic_id])
            .set(value);
    }

    pub fn update_topic_queue_size(&self, topic_id: &str, queue_id: &str, value: i64) {
        self.topic_queue_size
            .with_label_values(&[topic_id, queue_id])
            .set(value);
    }

    pub fn build(&self) -> Vec<u8> {
        let mut buffer = vec![];
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        encoder.encode(&metric_families, &mut buffer).unwrap();

        buffer
    }
}
