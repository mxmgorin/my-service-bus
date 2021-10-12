use std::{collections::HashMap, sync::Arc};

use my_service_bus_shared::date_time::{AtomicDateTimeAsMicroseconds, DateTimeAsMicroseconds};
use my_service_bus_tcp_shared::{PacketProtVer, TcpContract};
use tokio::{io::WriteHalf, net::TcpStream, sync::RwLock};

use crate::{app::AppContext, queue_subscribers::SubscriberId};

use super::{
    my_sb_session_data::ConnectedState, MySbSessionMetrics, MyServiceBusSessionData,
    SessionOperationError,
};

pub type ConnectionId = i64;

pub struct MyServiceBusSession {
    data: RwLock<MyServiceBusSessionData>,
    pub ip: String,
    pub id: ConnectionId,
    pub connected: DateTimeAsMicroseconds,
    pub last_incoming_package: AtomicDateTimeAsMicroseconds,
    pub app: Arc<AppContext>,
}

const BADGE_HIGHLIGHT_TIMOUT: u8 = 2;

pub struct SessionMetrics {
    pub name: Option<String>,
    pub version: Option<String>,
    pub ip: String,
    pub id: SubscriberId,
    pub metrics: MySbSessionMetrics,
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
        data.metrics.increase_read_size(read_size).await;
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
        }
    }

    async fn serialize_tcp_contract(&self, tcp_contract: TcpContract) -> Vec<u8> {
        let data = self.data.read().await;
        tcp_contract.serialize(&data.attr)
    }

    pub async fn send(&self, tcp_contract: TcpContract) -> Result<(), SessionOperationError> {
        let buf = self.serialize_tcp_contract(tcp_contract).await;

        let mut write_access = self.data.write().await;
        let result = write_access.send(&buf).await;

        result
    }

    pub async fn add_publisher(&self, topic: &str) {
        let mut data = self.data.write().await;

        data.metrics
            .publishers
            .insert(topic.to_string(), BADGE_HIGHLIGHT_TIMOUT);

        if !data.metrics.publishers.contains_key(topic) {
            data.metrics
                .publishers
                .insert(topic.to_string(), BADGE_HIGHLIGHT_TIMOUT);
        }
    }

    pub async fn disconnect(&self) -> Result<ConnectedState, ()> {
        let mut write_access = self.data.write().await;

        let result = write_access.disconnect().await;

        match result {
            Some(state) => Ok(state),
            None => Err(()),
        }
    }

    pub async fn get_packet_and_protocol_version(&self, packet: u8) -> PacketProtVer {
        let read_access = self.data.read().await;
        PacketProtVer {
            packet_version: read_access.attr.versions.get_packet_version(packet),
            protocol_version: read_access.attr.protocol_version,
        }
    }
}
