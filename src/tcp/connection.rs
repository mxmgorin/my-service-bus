use std::sync::Arc;

use my_service_bus_shared::queue_with_intervals::QueueWithIntervals;
use my_service_bus_tcp_shared::{ConnectionAttributes, TcpContract};

use crate::{app::AppContext, operations, sessions::MyServiceBusSession};

use super::error::MySbSocketError;

pub async fn on_disconnect(
    app: Arc<AppContext>,
    my_sb_session: Arc<MyServiceBusSession>,
) -> Result<(), String> {
    let result = tokio::task::spawn(on_disconnect_process(app.clone(), my_sb_session)).await;

    if let Err(err) = result {
        return Err(format!("{:?}", err));
    }

    Ok(())
}

async fn on_disconnect_process(app: Arc<AppContext>, my_sb_session: Arc<MyServiceBusSession>) {
    crate::operations::sessions::disconnect(app.as_ref(), my_sb_session.id).await;
}

pub async fn handle_incoming_payload(
    app: Arc<AppContext>,
    tcp_contract: TcpContract,
    session: &MyServiceBusSession,
    attr: &mut ConnectionAttributes,
) -> Result<(), MySbSocketError> {
    match tcp_contract {
        TcpContract::Ping {} => {
            session.send(TcpContract::Pong).await;
            Ok(())
        }
        TcpContract::Pong {} => Ok(()),
        TcpContract::Greeting {
            name,
            protocol_version,
        } => {
            attr.protocol_version = protocol_version;

            let splited: Vec<&str> = name.split(";").collect();

            println!("{:?}", splited);

            if splited.len() == 2 {
                session
                    .set_socket_name(splited[0].to_string(), Some(splited[1].to_string()))
                    .await;
            } else {
                session.set_socket_name(name, None).await;
            }

            session.set_protocol_version(protocol_version).await;
            Ok(())
        }
        TcpContract::Publish {
            topic_id,
            request_id,
            persist_immediately,
            data_to_publish,
        } => {
            session.topic_has_activity(topic_id.as_str()).await;

            let result = operations::publisher::publish(
                app,
                session,
                topic_id.as_str(),
                data_to_publish,
                persist_immediately,
            )
            .await;

            if let Err(err) = result {
                session
                    .send(TcpContract::Reject {
                        message: format!("{:?}", err),
                    })
                    .await;
            } else {
                session
                    .send(TcpContract::PublishResponse { request_id })
                    .await;
            }

            Ok(())
        }

        TcpContract::PublishResponse { request_id: _ } => {
            //This is a client packet
            Ok(())
        }
        TcpContract::Subscribe {
            topic_id,
            queue_id,
            queue_type,
        } => {
            operations::subscriber::subscribe_to_queue(
                app,
                topic_id.as_str(),
                queue_id.as_str(),
                queue_type,
                session,
            )
            .await?;
            Ok(())
        }
        TcpContract::SubscribeResponse {
            topic_id: _,
            queue_id: _,
        } => {
            //This is a client packet
            Ok(())
        }
        TcpContract::NewMessages(_) => {
            //This is a client packet
            Ok(())
        }
        TcpContract::NewMessagesConfirmation {
            topic_id,
            queue_id,
            confirmation_id,
        } => {
            operations::subscriber::confirm_delivery(
                app,
                topic_id.as_str(),
                queue_id.as_str(),
                session,
                confirmation_id,
            )
            .await?;
            Ok(())
        }
        TcpContract::CreateTopicIfNotExists { topic_id } => {
            operations::publisher::create_topic_if_not_exists(app, session, topic_id.as_str())
                .await;
            Ok(())
        }
        TcpContract::ConfirmMessagesByNotDelivery {
            packet_version: _,
            topic_id,
            queue_id,
            confirmation_id,
            not_delivered,
        } => {
            operations::subscriber::some_messages_are_not_confirmed(
                app,
                topic_id.as_str(),
                queue_id.as_str(),
                confirmation_id,
                QueueWithIntervals::restore(not_delivered),
            )
            .await?;

            Ok(())
        }
        TcpContract::PacketVersions { packet_versions } => {
            attr.versions.update(&packet_versions);
            session.update_packet_versions(&packet_versions).await;
            Ok(())
        }
        TcpContract::Reject { message: _ } => {
            //This is a client packet
            Ok(())
        }
        TcpContract::AllMessagesConfirmedAsFail {
            topic_id,
            queue_id,
            confirmation_id,
        } => {
            operations::subscriber::confirm_non_delivery(
                app,
                topic_id.as_str(),
                queue_id.as_str(),
                session,
                confirmation_id,
            )
            .await?;
            Ok(())
        }

        TcpContract::ConfirmSomeMessagesAsOk {
            packet_version: _,
            topic_id,
            queue_id,
            confirmation_id,
            delivered,
        } => {
            operations::subscriber::some_messages_are_confirmed(
                app,
                topic_id.as_str(),
                queue_id.as_str(),
                confirmation_id,
                QueueWithIntervals::restore(delivered),
            )
            .await?;

            Ok(())
        }
    }
}
