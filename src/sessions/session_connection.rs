use std::sync::Arc;

use my_service_bus_tcp_shared::{MySbTcpSerializer, TcpContract};
use my_tcp_sockets::tcp_connection::SocketConnection;

pub enum SessionConnection {
    Tcp(Arc<SocketConnection<TcpContract, MySbTcpSerializer>>),
}

impl SessionConnection {
    pub fn get_id(&self) -> i32 {
        match self {
            SessionConnection::Tcp(connection) => connection.id,
        }
    }

    pub fn get_ip(&self) -> &str {
        todo!("Implement")
    }
}
