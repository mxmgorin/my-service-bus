use rust_extensions::date_time::DateTimeAsMicroseconds;
use tokio::sync::Mutex;

use crate::sessions::{ConnectionMetrics, ConnectionMetricsSnapshot};

pub struct HttpConnectionData {
    pub id: String,
    pub name: String,
    pub version: String,
    pub ip: String,
    pub connected: DateTimeAsMicroseconds,
    connection_metrics: ConnectionMetrics,
    pub disconnected: Mutex<DateTimeAsMicroseconds>,
}

impl HttpConnectionData {
    pub fn new(id: String, name: String, version: String, ip: String) -> Self {
        Self {
            id,
            name,
            version,
            ip,
            disconnected: Mutex::new(DateTimeAsMicroseconds::now()),
            connection_metrics: ConnectionMetrics::new(),
            connected: DateTimeAsMicroseconds::now(),
        }
    }

    pub fn ping(&self) {
        self.connection_metrics.add_read(1);
    }

    pub fn get_connection_metrics(&self) -> ConnectionMetricsSnapshot {
        return self.connection_metrics.get_snapshot();
    }

    pub fn one_second_tick(&self) {
        self.connection_metrics.one_second_tick();
    }
}
