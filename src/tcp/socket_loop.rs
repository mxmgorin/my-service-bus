use std::sync::Arc;

use my_service_bus_tcp_shared::{MySbTcpSerializer, TcpContract};
use my_tcp_sockets::tcp_connection::{ConnectionCallback, ConnectionEvent};

use crate::{
    app::{logs::SystemProcess, AppContext},
    sessions::{MyServiceBusSession, SessionConnection},
};

pub async fn start(
    mut socket_reader: ConnectionCallback<TcpContract, MySbTcpSerializer>,
    app: Arc<AppContext>,
) {
    loop {
        match socket_reader.get_next_event().await {
            ConnectionEvent::Connected(connection) => {
                let session =
                    MyServiceBusSession::new(SessionConnection::Tcp(connection), app.clone());
                app.sessions.add(Arc::new(session)).await;
            }
            ConnectionEvent::Disconnected(connection) => {
                if let Some(session) = app.sessions.remove(&connection.id).await {
                    crate::operations::sessions::disconnect(app.as_ref(), session.as_ref()).await;
                }
            }
            ConnectionEvent::Payload {
                connection,
                payload,
            } => {
                let connection_id = connection.id;
                if let Err(err) =
                    super::icoming_packets::handle(app.clone(), payload, connection).await
                {
                    app.logs.add_error(
                        None,
                        SystemProcess::TcpSocket,
                        "Handle Payload".to_string(),
                        format!("Err: {:?}", err),
                        Some(format!("ConnectionId:{}", connection_id)),
                    );
                }
            }
        }
    }
}
