use std::sync::Arc;

use my_service_bus_tcp_shared::TcpContract;

use crate::{app::AppContext, sessions::MyServiceBusSession, tcp::tcp_server::ConnectionId};

pub async fn disconnect(app: &AppContext, session_id: i64) -> Option<Arc<MyServiceBusSession>> {
    let result = app.sessions.remove(&session_id).await;

    if let Some(removed_session) = &result {
        removed_session.disconnect().await;
        handle_after_disconnect(app, removed_session.as_ref()).await
    }

    result
}

pub async fn send_package(
    app: &AppContext,
    connection_id: ConnectionId,
    tcp_contract: TcpContract,
) -> bool {
    let session = app.sessions.get(connection_id).await;

    if session.is_none() {
        return false;
    }

    let session = session.unwrap();

    if !session.send(tcp_contract).await {
        disconnect(app, connection_id).await;
    }

    return true;
}

async fn handle_after_disconnect(app: &AppContext, removed_session: &MyServiceBusSession) {
    let topics = app.topic_list.get_all().await;

    for topic in &topics {
        let mut topic_data = topic.get_access("handle_after_disconnect").await;

        let removed_subscribers = topic_data.disconnect(removed_session.id);

        if let Some(removed_subscribers) = removed_subscribers {
            for (topic_queue, removed_subscriber) in removed_subscribers {
                println!(
                    "Subscriber {} with connection_id {} is removed during the session [{}]/{} disconnect process",
                    removed_subscriber.id,
                    removed_subscriber.session_id,
                    removed_session.id,
                    removed_session.get_name().await
                );
                crate::operations::subscriber::remove_subscriber(topic_queue, removed_subscriber);
            }
        }
    }
}
