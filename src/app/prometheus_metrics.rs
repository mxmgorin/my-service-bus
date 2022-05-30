use prometheus::{Encoder, IntGauge, IntGaugeVec, Opts, Registry, TextEncoder};

pub struct PrometheusMetrics {
    registry: Registry,
    pub persist_queue_size: IntGaugeVec,
    pub topic_queue_size: IntGaugeVec,
    permanent_queues_without_subscribers: IntGauge,
    topics_without_queues: IntGauge,
}

impl PrometheusMetrics {
    pub fn new() -> Self {
        let registry = Registry::new();

        // This unwraps runs on application start. If it fails - applications is not started
        let persist_queue_size = create_topic_persist_queue_size();

        let permanent_queues_without_subscribers = create_permanent_queues_without_subscribers();

        let topic_queue_size = create_topic_queue_size();

        let topics_without_queues = create_topics_without_queues();

        registry
            .register(Box::new(topic_queue_size.clone()))
            .unwrap();

        registry
            .register(Box::new(persist_queue_size.clone()))
            .unwrap();

        registry
            .register(Box::new(permanent_queues_without_subscribers.clone()))
            .unwrap();

        registry
            .register(Box::new(topics_without_queues.clone()))
            .unwrap();

        return Self {
            registry,
            persist_queue_size,
            topic_queue_size,
            permanent_queues_without_subscribers,
            topics_without_queues,
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

    pub fn update_permanent_queues_without_subscribers(&self, value: i64) {
        self.permanent_queues_without_subscribers.set(value);
    }

    pub fn update_topics_without_queues(&self, value: i64) {
        self.topics_without_queues.set(value);
    }

    pub fn build(&self) -> Vec<u8> {
        let mut buffer = vec![];
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        encoder.encode(&metric_families, &mut buffer).unwrap();

        buffer
    }
}

fn create_topic_persist_queue_size() -> IntGaugeVec {
    let gauge_opts = Opts::new("topic_persist_queue_size", "Topic queue to persist size");

    let lables = &["topic"];

    IntGaugeVec::new(gauge_opts, lables).unwrap()
}

fn create_topic_queue_size() -> IntGaugeVec {
    let gauge_opts = Opts::new("topic_queue_size", "Topic queue size");

    let lables = &["topic", "queue"];

    // This unwraps runs on application start. If it fails - applications is not started
    IntGaugeVec::new(gauge_opts, lables).unwrap()
}

fn create_permanent_queues_without_subscribers() -> IntGauge {
    IntGauge::new(
        "permanent_queues_without_subscribers",
        "Permanent queues without subscribers count",
    )
    .unwrap()
}

fn create_topics_without_queues() -> IntGauge {
    IntGauge::new("topics_without_queues", "Topics without queues").unwrap()
}
