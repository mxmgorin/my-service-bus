use std::sync::Arc;

use my_service_bus_tcp_shared::TcpContract;

use crate::{
    app::AppContext,
    sessions::{MyServiceBusSession, SessionOperationError},
};

pub async fn disconnect(app: &AppContext, session: Arc<MyServiceBusSession>) {
    let disconnect_result = session.disconnect().await;

    if let Ok(_) = disconnect_result {
        handle_after_disconnect(app, session.as_ref()).await
    }
}

async fn handle_after_disconnect(app: &AppContext, session: &MyServiceBusSession) {
    let topics = app.topic_list.get_all().await;

    for topic in &topics {
        let removed_subscribers = topic
            .queues
            .remove_subscribers_by_connection_id(session.id)
            .await;

        for removed_subscriber in removed_subscribers {
            println!(
                "Subscriber {} with connection_id {} is removed during the session [{}]/{} disconnect process",
                removed_subscriber.id,
                removed_subscriber.session.id,
                session.id,
                session.get_name().await
            );
            crate::operations::subscriber::handle_subscriber_remove(removed_subscriber).await;
        }
    }
}

pub async fn send_package(
    app: &AppContext,
    session: &MyServiceBusSession,
    tcp_contract: TcpContract,
) {
    let result = session.send(tcp_contract).await;

    if let Err(err) = result {
        if let SessionOperationError::JustDisconnected = err {
            handle_after_disconnect(app, session).await;
        }
    }
}
