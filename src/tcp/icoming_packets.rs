use std::sync::Arc;

use my_service_bus_shared::queue_with_intervals::QueueWithIntervals;
use my_service_bus_tcp_shared::{MySbTcpSerializer, TcpContract};
use my_tcp_sockets::tcp_connection::SocketConnection;

use crate::{app::AppContext, operations};

use super::error::MySbSocketError;

pub async fn handle(
    app: Arc<AppContext>,
    tcp_contract: TcpContract,
    connection: Arc<SocketConnection<TcpContract, MySbTcpSerializer>>,
) -> Result<(), MySbSocketError> {
    match tcp_contract {
        TcpContract::Ping {} => {
            connection
                .send_bytes(TcpContract::Pong.serialize().as_ref())
                .await;
            Ok(())
        }
        TcpContract::Pong {} => Ok(()),
        TcpContract::Greeting {
            name,
            protocol_version: _,
        } => {
            //TODO - It Should be scan from the last to ;
            let splited: Vec<&str> = name.split(";").collect();

            if let Some(session) = app.sessions.get(connection.id).await {
                if splited.len() == 2 {
                    session
                        .set_socket_name(splited[0].to_string(), Some(splited[1].to_string()))
                        .await;
                } else {
                    session.set_socket_name(name, None).await;
                }
            }

            Ok(())
        }
        TcpContract::Publish {
            topic_id,
            request_id,
            persist_immediately,
            data_to_publish,
        } => {
            let result = operations::publisher::publish(
                app.clone(),
                topic_id,
                data_to_publish,
                persist_immediately,
                connection.id,
            )
            .await;

            if let Err(err) = result {
                connection
                    .send(TcpContract::Reject {
                        message: format!("{:?}", err),
                    })
                    .await;
            } else {
                connection
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
            let delivery_version_id = {
                let socket_data = connection.socket.lock().await;

                if let Some(socket_data) = &*socket_data {
                    Some(
                        socket_data
                            .get_serializer()
                            .get_new_messages_packet_version(),
                    )
                } else {
                    None
                }
            };

            if let Some(delivery_version_id) = delivery_version_id {
                operations::subscriber::subscribe_to_queue(
                    app,
                    topic_id,
                    queue_id,
                    queue_type,
                    connection.id,
                    delivery_version_id,
                )
                .await?;
            }

            Ok(())
        }
        TcpContract::SubscribeResponse {
            topic_id: _,
            queue_id: _,
        } => {
            //This is a client packet
            Ok(())
        }
        TcpContract::NewMessagesServerSide(_) => {
            //This is a client packet
            Ok(())
        }
        TcpContract::NewMessagesConfirmation {
            topic_id,
            queue_id,
            confirmation_id,
        } => {
            operations::delivery_confirmation::all_confirmed(
                app,
                topic_id.as_str(),
                queue_id.as_str(),
                confirmation_id,
            )
            .await?;
            Ok(())
        }
        TcpContract::CreateTopicIfNotExists { topic_id } => {
            if let Some(session) = app.sessions.get(connection.id).await {
                operations::publisher::create_topic_if_not_exists(
                    app,
                    Some(session.as_ref()),
                    topic_id.as_str(),
                )
                .await;
            }

            Ok(())
        }
        TcpContract::IntermediaryConfirm {
            packet_version: _,
            topic_id,
            queue_id,
            confirmation_id,
            delivered,
        } => {
            operations::delivery_confirmation::intermediary_confirm(
                app,
                topic_id.as_str(),
                queue_id.as_str(),
                confirmation_id,
                QueueWithIntervals::restore(delivered),
            )
            .await?;

            Ok(())
        }
        TcpContract::PacketVersions { packet_versions: _ } => {
            //This is a serializer layer packet
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
            operations::delivery_confirmation::all_fail(
                app,
                topic_id.as_str(),
                queue_id.as_str(),
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
            operations::delivery_confirmation::some_messages_are_confirmed(
                app,
                topic_id.as_str(),
                queue_id.as_str(),
                confirmation_id,
                QueueWithIntervals::restore(delivered),
            )
            .await?;

            Ok(())
        }
        TcpContract::NewMessages {
            topic_id: _,
            queue_id: _,
            confirmation_id: _,
            messages: _,
        } => {
            //this is Client Side Message

            Ok(())
        }
    }
}
