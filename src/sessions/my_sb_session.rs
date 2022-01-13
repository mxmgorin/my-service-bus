use rust_extensions::date_time::DateTimeAsMicroseconds;
use tokio::sync::RwLock;

use super::{ConnectionMetricsSnapshot, MyServiceBusSessionData, SessionConnection, SessionId};
pub struct SessionMetrics {
    pub name: Option<String>,
    pub version: Option<String>,
    pub ip: String,
    pub id: SessionId,
    pub connection_metrics: ConnectionMetricsSnapshot,
    pub protocol_version: String,
}

pub struct MyServiceBusSession {
    pub id: SessionId,
    data: RwLock<MyServiceBusSessionData>,
    pub connection: SessionConnection,
    pub connected: DateTimeAsMicroseconds,
}

impl MyServiceBusSession {
    pub fn new(id: SessionId, connection: SessionConnection) -> Self {
        let data = MyServiceBusSessionData::new();
        Self {
            connection,
            data: RwLock::new(data),
            id,
            connected: DateTimeAsMicroseconds::now(),
        }
    }

    pub async fn set_socket_name(&self, set_socket_name: String, client_version: Option<String>) {
        let mut data = self.data.write().await;
        data.name = Some(set_socket_name);
        data.client_version = client_version;
    }

    pub fn update_tcp_protocol_version(&self, value: i32) {
        if let SessionConnection::Tcp(connection_data) = &self.connection {
            connection_data.update_protocol_version(value);
        } else {
            panic!(
                "Invalid connection type  [{}] to update Tcp protocol version",
                self.connection.get_connection_type()
            );
        }
    }

    pub fn update_tcp_delivery_packet_version(&self, value: i32) {
        if let SessionConnection::Tcp(connection_data) = &self.connection {
            connection_data.update_deliver_message_packet_version(value);
        } else {
            panic!(
                "Invalid connection type  [{}] to update Tcp delivery packet version",
                self.connection.get_connection_type()
            );
        }
    }

    pub async fn get_name(&self) -> String {
        let data = self.data.read().await;

        let result = match &data.name {
            Some(name) => format!("{} {}", name, self.connection.get_ip()),
            None => self.connection.get_ip().to_string(),
        };
        result
    }

    fn get_protocol_version(&self) -> String {
        match &self.connection {
            SessionConnection::Tcp(data) => format!("Tcp: {}", data.get_protocol_version()),
            SessionConnection::Http(_) => "Http".to_string(),
            #[cfg(test)]
            SessionConnection::Test(_) => "Test".to_string(),
        }
    }

    pub async fn get_metrics(&self) -> SessionMetrics {
        let connection_metrics = match &self.connection {
            SessionConnection::Tcp(data) => data.get_connection_metrics(),
            SessionConnection::Http(data) => data.get_connection_metrics(),
            #[cfg(test)]
            SessionConnection::Test(_) => {
                panic!("We do not have metrics in test enviroment");
            }
        };

        let protocol_version = self.get_protocol_version();

        let read_access = self.data.read().await;

        SessionMetrics {
            id: self.id,
            name: read_access.get_name(),
            version: read_access.get_version(),
            ip: self.connection.get_ip().to_string(),
            protocol_version,
            connection_metrics,
        }
    }

    pub async fn disconnect(&self) -> bool {
        match &self.connection {
            SessionConnection::Tcp(data) => {
                return data.connection.disconnect().await;
            }
            SessionConnection::Http(data) => {
                return data.disconnect();
            }
            #[cfg(test)]
            SessionConnection::Test(connection) => {
                let result = connection
                    .connected
                    .load(std::sync::atomic::Ordering::SeqCst);

                if result == false {
                    return false;
                }

                connection
                    .connected
                    .store(false, std::sync::atomic::Ordering::SeqCst);

                return true;
            }
        }
    }
}
