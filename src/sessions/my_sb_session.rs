use std::sync::Arc;

use rust_extensions::date_time::DateTimeAsMicroseconds;
use tokio::sync::RwLock;

use crate::app::AppContext;

use super::{ConnectionMetrics, MyServiceBusSessionData, SessionConnection, SessionId};

pub struct MyServiceBusSession {
    pub id: i32,
    data: RwLock<MyServiceBusSessionData>,
    pub connection: SessionConnection,
    pub app: Arc<AppContext>,
    pub connected: DateTimeAsMicroseconds,
}

pub struct SessionMetrics {
    pub name: Option<String>,
    pub version: Option<String>,
    pub ip: String,
    pub id: SessionId,
    pub protocol_version: i32,
    pub connection_metrics: ConnectionMetrics,
}

impl MyServiceBusSession {
    pub fn new(connection: SessionConnection, app: Arc<AppContext>) -> Self {
        let data = MyServiceBusSessionData::new(app.clone());
        let id = connection.get_id();
        Self {
            connection,
            data: RwLock::new(data),
            id,
            app,
            connected: DateTimeAsMicroseconds::now(),
        }
    }

    pub async fn set_socket_name(&self, set_socket_name: String, client_version: Option<String>) {
        let mut data = self.data.write().await;
        data.name = Some(set_socket_name);
        data.client_version = client_version;
    }

    pub async fn get_name(&self) -> String {
        let data = self.data.read().await;

        let result = match &data.name {
            Some(name) => format!("{} {}", name, self.connection.get_ip()),
            None => self.connection.get_ip().to_string(),
        };
        result
    }

    pub async fn get_metrics(&self) -> SessionMetrics {
        let connection_metrics = match &self.connection {
            SessionConnection::Tcp(connection) => {
                ConnectionMetrics::from_tcp(&connection.statistics)
            }
        };

        let read_access = self.data.read().await;

        SessionMetrics {
            id: self.connection.get_id(),
            name: read_access.get_name(),
            version: read_access.get_version(),
            ip: self.connection.get_ip().to_string(),
            protocol_version: read_access.attr.protocol_version,
            connection_metrics,
        }
    }

    pub fn get_last_incoming_package_moment(&self) -> DateTimeAsMicroseconds {
        match &self.connection {
            SessionConnection::Tcp(tcp) => tcp.statistics.last_receive_moment.as_date_time(),
        }
    }

    pub async fn disconnect(&self) -> bool {
        match &self.connection {
            SessionConnection::Tcp(tcp) => tcp.disconnect().await,
        }
    }
}
