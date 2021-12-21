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
        }
    }

    pub fn increase_read_size(&mut self, read_size: usize) {
        self.read_size += read_size;
        self.read_per_sec_going.increase(read_size);
    }

    pub fn increase_written_size(&mut self, written_size: usize) {
        self.written_size += written_size;
        self.written_per_sec_going.increase(written_size);
    }

    pub fn one_second_tick(&mut self) {
        self.read_per_sec = self.read_per_sec_going.get_and_reset();
        self.written_per_sec = self.written_per_sec_going.get_and_reset();
    }
}
