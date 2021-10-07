use std::collections::HashMap;

use crate::metric_data::MetricOneSecond;

#[derive(Clone)]
pub struct MySbSessionMetrics {
    pub disconnected: bool,
    pub read_size: usize,
    pub written_size: usize,

    pub read_per_sec: usize,
    pub written_per_sec: usize,

    pub read_per_sec_going: MetricOneSecond,
    pub written_per_sec_going: MetricOneSecond,

    pub publishers: HashMap<String, u8>,

    pub active: u8,
}

impl MySbSessionMetrics {
    pub fn new() -> Self {
        Self {
            disconnected: false,
            read_size: 0,
            written_size: 0,
            read_per_sec: 0,
            written_per_sec: 0,
            read_per_sec_going: MetricOneSecond::new(),
            written_per_sec_going: MetricOneSecond::new(),
            publishers: HashMap::new(),

            active: 0,
        }
    }

    pub async fn increase_read_size(&mut self, read_size: usize) {
        self.read_size += read_size;
        self.read_per_sec_going.increase(read_size);
    }

    pub async fn increase_written_size(&mut self, written_size: usize) {
        self.written_size += written_size;
        self.written_per_sec_going.increase(written_size);
    }

    pub fn one_second_tick(&mut self) {
        self.read_per_sec = self.read_per_sec_going.get_and_reset();
        self.written_per_sec = self.written_per_sec_going.get_and_reset();

        let topics: Vec<String> = self
            .publishers
            .keys()
            .into_iter()
            .map(|itm| itm.to_string())
            .collect();

        for topic_id in topics {
            let value = self.publishers.get(topic_id.as_str());

            if value.is_none() {
                println!(
                    "one_second_tick: Somehow we can not get publishers for topic {}.",
                    topic_id
                );
                continue;
            };

            let value = value.unwrap().clone();

            if value > 0 {
                self.publishers.insert(topic_id, value - 1);
            }
        }
    }
}
