use std::time::Duration;

use my_service_bus_tcp_shared::{MySbTcpSerializer, TcpContract};
use my_tcp_sockets::tcp_connection::SocketConnection;

pub async fn send_with_timeout(
    connection: &SocketConnection<TcpContract, MySbTcpSerializer>,
    tcp_contract: TcpContract,
) {
    let timeout = Duration::from_secs(5);

    match tokio::time::timeout(timeout, connection.send(tcp_contract)).await {
        Ok(_) => {
            return;
        }
        Err(err) => {
            println!(
                "Socket {} is timeouted with err: {:?}",
                connection.connection_name.get().await,
                err
            );

            connection.disconnect().await;
        }
    }
}
