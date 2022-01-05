use std::sync::atomic::Ordering;

use my_service_bus_tcp_shared::{MySbTcpSerializer, TcpContract};
use my_tcp_sockets::tcp_connection::SocketConnection;

pub struct ConnectionMetrics {
    pub read: usize,
    pub written: usize,
    pub read_per_sec: usize,
    pub written_per_sec: usize,
}

impl ConnectionMetrics {
    pub fn from_tcp(connection: &SocketConnection<TcpContract, MySbTcpSerializer>) -> Self {
        Self {
            read: connection.statistics.total_received.load(Ordering::SeqCst),
            written: connection.statistics.total_sent.load(Ordering::SeqCst),
            read_per_sec: connection.statistics.received_per_sec.get_value(),
            written_per_sec: connection.statistics.sent_per_sec.get_value(),
        }
    }
}
