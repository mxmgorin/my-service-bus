use std::sync::{
    atomic::{AtomicI32, Ordering},
    Arc,
};

use my_service_bus_tcp_shared::{MySbTcpSerializer, PacketProtVer, TcpContract};
use my_tcp_sockets::tcp_connection::SocketConnection;

use crate::sessions::ConnectionMetricsSnapshot;

pub struct TcpConnectionData {
    pub connection: Arc<SocketConnection<TcpContract, MySbTcpSerializer>>,
    protocol_version: AtomicI32,
    delivery_packet_version: AtomicI32,
}

impl TcpConnectionData {
    pub fn new(connection: Arc<SocketConnection<TcpContract, MySbTcpSerializer>>) -> Self {
        Self {
            connection,
            protocol_version: AtomicI32::new(0),
            delivery_packet_version: AtomicI32::new(0),
        }
    }

    pub fn update_protocol_version(&self, value: i32) {
        self.protocol_version.store(value, Ordering::SeqCst);
    }

    pub fn update_deliver_message_packet_version(&self, value: i32) {
        self.delivery_packet_version.store(value, Ordering::SeqCst);
    }

    pub fn get_protocol_version(&self) -> i32 {
        self.protocol_version.load(Ordering::Relaxed)
    }

    pub fn get_messages_to_deliver_protocol_version(&self) -> PacketProtVer {
        let protocol_version = self.get_protocol_version();
        if protocol_version == 0 {
            panic!("Protocol version is not initialized");
        }
        let packet_version = self.delivery_packet_version.load(Ordering::Relaxed);

        PacketProtVer {
            protocol_version,
            packet_version,
        }
    }

    pub fn get_connection_metrics(&self) -> ConnectionMetricsSnapshot {
        ConnectionMetricsSnapshot {
            read: self
                .connection
                .statistics
                .total_received
                .load(Ordering::SeqCst),
            written: self.connection.statistics.total_sent.load(Ordering::SeqCst),
            read_per_sec: self.connection.statistics.received_per_sec.get_value(),
            written_per_sec: self.connection.statistics.sent_per_sec.get_value(),
            last_incoming_moment: self
                .connection
                .statistics
                .last_receive_moment
                .as_date_time(),
        }
    }
}
