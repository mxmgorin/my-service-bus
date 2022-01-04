use std::sync::atomic::Ordering;

use my_tcp_sockets::tcp_connection::ConnectionStatistics;

pub struct ConnectionMetrics {
    pub read: usize,
    pub written: usize,
    pub read_per_sec: usize,
    pub written_per_sec: usize,
}

impl ConnectionMetrics {
    pub fn from_tcp(statistics: &ConnectionStatistics) -> Self {
        Self {
            read: statistics.total_received.load(Ordering::SeqCst),
            written: statistics.total_sent.load(Ordering::SeqCst),
            read_per_sec: statistics.received_per_sec.get_value(),
            written_per_sec: statistics.sent_per_sec.get_value(),
        }
    }
}
