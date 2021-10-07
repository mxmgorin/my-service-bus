use std::{collections::HashMap, sync::Arc};

use my_service_bus_shared::date_time::{AtomicDateTimeAsMicroseconds, DateTimeAsMicroseconds};
use my_service_bus_tcp_shared::TcpContract;
use tokio::{io::WriteHalf, net::TcpStream, sync::RwLock};

use crate::app::AppContext;

use super::{my_sb_session_data::ConnectedState, MyServiceBusSessionData, SessionOperationError};

pub type ConnectionId = i64;

pub struct MyServiceBusSession {
    pub data: RwLock<MyServiceBusSessionData>,
    pub ip: String,
    pub id: ConnectionId,
    pub connected: DateTimeAsMicroseconds,
    pub last_incoming_package: AtomicDateTimeAsMicroseconds,
    pub app: Arc<AppContext>,
}

const BADGE_HIGHLIGHT_TIMOUT: u8 = 2;

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

    pub async fn increase_read_size(&self, process_id: i64, read_size: usize) {
        self.app
            .enter_lock(
                process_id,
                format!("MySbSession[{}].increase_read_size", self.id),
            )
            .await;
        let mut data = self.data.write().await;
        data.statistic.increase_read_size(read_size).await;

        self.app.exit_lock(process_id).await;
    }

    pub async fn set_socket_name(
        &self,
        process_id: i64,
        set_socket_name: String,
        client_version: Option<String>,
    ) {
        self.app
            .enter_lock(
                process_id,
                format!("MySbSession[{}].set_socket_name", self.id),
            )
            .await;

        let mut data = self.data.write().await;
        data.name = Some(set_socket_name);
        data.client_version = client_version;

        self.app.exit_lock(process_id).await;
    }

    pub async fn set_protocol_version(&self, process_id: i64, protocol_version: i32) {
        self.app
            .enter_lock(
                process_id,
                format!("MySbSession[{}].set_protocol_version", self.id),
            )
            .await;

        let mut data = self.data.write().await;
        data.attr.protocol_version = protocol_version;

        self.app.exit_lock(process_id).await;
    }

    pub async fn update_packet_versions(
        &self,
        process_id: i64,
        packet_versions: &HashMap<u8, i32>,
    ) {
        self.app
            .enter_lock(
                process_id,
                format!("MySbSession[{}].update_packet_versions", self.id),
            )
            .await;
        let mut data = self.data.write().await;
        data.attr.versions.update(packet_versions);
        self.app.exit_lock(process_id).await;
    }

    pub async fn one_second_tick(&self, process_id: i64) {
        self.app
            .enter_lock(
                process_id,
                format!("MySbSession[{}].one_second_tick", self.id),
            )
            .await;
        let mut write_access = self.data.write().await;
        write_access.statistic.one_second_tick();

        self.app.exit_lock(process_id).await;
    }

    pub async fn get_name(&self, process_id: i64) -> String {
        self.app
            .enter_lock(process_id, format!("MySbSession[{}].get_name", self.id))
            .await;

        let data = self.data.read().await;

        let result = match &data.name {
            Some(name) => format!("{} {}", name, self.ip),
            None => self.ip.clone(),
        };

        self.app.exit_lock(process_id).await;

        result
    }

    async fn serialize_tcp_contract(&self, tcp_contract: TcpContract) -> Vec<u8> {
        let data = self.data.read().await;
        tcp_contract.serialize(&data.attr)
    }

    pub async fn send(
        &self,
        process_id: i64,
        tcp_contract: TcpContract,
    ) -> Result<(), SessionOperationError> {
        let buf = self.serialize_tcp_contract(tcp_contract).await;

        self.app
            .enter_lock(process_id, format!("MySbSession[{}].send", self.id))
            .await;

        let mut write_access = self.data.write().await;
        let result = write_access.send(&buf).await;
        self.app.exit_lock(process_id).await;

        result
    }

    pub async fn add_publisher(&self, process_id: i64, topic: &str) {
        self.app
            .enter_lock(
                process_id,
                format!("MySbSession[{}].add_publisher", self.id),
            )
            .await;
        let mut data = self.data.write().await;

        data.statistic
            .publishers
            .insert(topic.to_string(), BADGE_HIGHLIGHT_TIMOUT);

        if !data.statistic.publishers.contains_key(topic) {
            data.statistic
                .publishers
                .insert(topic.to_string(), BADGE_HIGHLIGHT_TIMOUT);
        }

        self.app.exit_lock(process_id).await;
    }

    pub async fn disconnect(&self, process_id: i64) -> Result<ConnectedState, ()> {
        self.app
            .enter_lock(process_id, format!("MySbSession[{}].disconnect", self.id))
            .await;

        let mut write_access = self.data.write().await;

        let result = write_access.disconnect().await;

        self.app.exit_lock(process_id).await;

        match result {
            Some(state) => Ok(state),
            None => Err(()),
        }
    }
}
