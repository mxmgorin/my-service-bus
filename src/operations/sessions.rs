use std::sync::Arc;

use my_service_bus_tcp_shared::TcpContract;

use crate::{
    app::AppContext,
    sessions::{MyServiceBusSession, SessionOperationError},
};

pub async fn disconnect(process_id: i64, app: &AppContext, session: Arc<MyServiceBusSession>) {
    let disconnect_result = session.disconnect(process_id).await;

    if let Ok(_) = disconnect_result {
        handle_after_disconnect(process_id, app, session.as_ref()).await
    }
}

async fn handle_after_disconnect(process_id: i64, app: &AppContext, session: &MyServiceBusSession) {
    let topics = app.topic_list.get_all().await;

    for topic in &topics {
        let removed_subscribers = topic
            .queues
            .remove_subscribers_by_connection_id(session.id)
            .await;

        for removed_subscriber in removed_subscribers {
            crate::operations::subscriber::handle_subscriber_remove(
                process_id,
                app,
                removed_subscriber,
            )
            .await;
        }
    }
}

pub async fn send_package(
    process_id: i64,
    app: &AppContext,
    session: &MyServiceBusSession,
    tcp_contract: TcpContract,
) {
    let result = session.send(process_id, tcp_contract).await;

    if let Err(err) = result {
        if let SessionOperationError::JustDisconnected = err {
            handle_after_disconnect(process_id, app, session).await;
        }
    }
}
