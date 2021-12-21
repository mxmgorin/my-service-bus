use std::{collections::HashMap, sync::Arc};

use my_service_bus_tcp_shared::TcpContract;
use rust_extensions::date_time::{AtomicDateTimeAsMicroseconds, DateTimeAsMicroseconds};
use tokio::{io::WriteHalf, net::TcpStream, sync::RwLock};

use crate::{app::AppContext, queue_subscribers::SubscriberId};

use super::{MySbSessionMetrics, MyServiceBusSessionData};

pub type ConnectionId = i64;

pub struct MyServiceBusSession {
    data: RwLock<MyServiceBusSessionData>,
    pub ip: String,
    pub id: ConnectionId,
    pub connected: DateTimeAsMicroseconds,
    pub last_incoming_package: AtomicDateTimeAsMicroseconds,
    pub app: Arc<AppContext>,
}

pub struct SessionMetrics {
    pub name: Option<String>,
    pub version: Option<String>,
    pub ip: String,
    pub id: SubscriberId,
    pub metrics: MySbSessionMetrics,
    pub protocol_version: i32,
}

impl MyServiceBusSession {
    pub fn new(
        id: ConnectionId,
        ip: String,
        tcp_stream: WriteHalf<TcpStream>,
        app: Arc<AppContext>,
    ) -> Self {
        let data = MyServiceBusSessionData::new(tcp_stream, app.clone());

        Self {
            id,
            ip,
            data: RwLock::new(data),
            connected: DateTimeAsMicroseconds::now(),
            last_incoming_package: AtomicDateTimeAsMicroseconds::now(),
            app,
        }
    }

    pub async fn increase_read_size(&self, read_size: usize) {
        let mut data = self.data.write().await;
        data.metrics.increase_read_size(read_size);
    }

    pub async fn set_socket_name(&self, set_socket_name: String, client_version: Option<String>) {
        let mut data = self.data.write().await;
        data.name = Some(set_socket_name);
        data.client_version = client_version;
    }

    pub async fn set_protocol_version(&self, protocol_version: i32) {
        let mut data = self.data.write().await;
        data.attr.protocol_version = protocol_version;
    }

    pub async fn update_packet_versions(&self, packet_versions: &HashMap<u8, i32>) {
        let mut data = self.data.write().await;
        data.attr.versions.update(packet_versions);
    }

    pub async fn one_second_tick(&self) {
        let mut write_access = self.data.write().await;
        write_access.metrics.one_second_tick();
    }

    pub async fn get_name(&self) -> String {
        let data = self.data.read().await;

        let result = match &data.name {
            Some(name) => format!("{} {}", name, self.ip),
            None => self.ip.clone(),
        };
        result
    }

    pub async fn get_metrics(&self) -> SessionMetrics {
        let read_access = self.data.read().await;

        SessionMetrics {
            id: self.id,
            name: read_access.get_name(),
            version: read_access.get_version(),
            ip: self.ip.to_string(),
            metrics: read_access.metrics.clone(),
            protocol_version: read_access.attr.protocol_version,
        }
    }

    pub async fn send(&self, tcp_contract: TcpContract) -> bool {
        let buf = tcp_contract.serialize();

        let mut write_access = self.data.write().await;
        return write_access.send(&buf).await;
    }

    pub async fn disconnect(&self) {
        let mut write_access = self.data.write().await;

        write_access.disconnect().await;
    }

    pub async fn get_packet_version(&self, packet: u8) -> i32 {
        let read_access = self.data.read().await;
        read_access.attr.versions.get_packet_version(packet)
    }
}
