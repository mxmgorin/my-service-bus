use std::sync::Arc;

use my_service_bus_shared::queue::TopicQueueType;

use crate::{
    app::AppContext,
    queue_subscribers::{QueueSubscriber, SubscribeErrorResult},
    sessions::MyServiceBusSession,
};

use super::OperationFailResult;

pub async fn subscribe_to_queue(
    process_id: i64,
    app: Arc<AppContext>,
    topic_id: &str,
    queue_id: &str,
    queue_type: TopicQueueType,
    session: Arc<MyServiceBusSession>,
) -> Result<(), OperationFailResult> {
    let mut topic = app.topic_list.get(topic_id).await;

    if topic.is_none() {
        if app.auto_create_topic_on_subscribe {
            topic = Some(app.topic_list.add_if_not_exists(topic_id).await);
        } else {
            return Err(OperationFailResult::TopicNotFound {
                topic_id: topic_id.to_string(),
            });
        }
    }

    let topic = topic.unwrap();

    let topic_queue = topic
        .queues
        .add_queue_if_not_exists(topic.topic_id.as_str(), queue_id, queue_type.clone())
        .await;

    let kicked_subscriber_result;

    let subscriber_id = app.subscriber_id_generator.get_next_subsriber_id();
    {
        let mut write_access = topic_queue
            .get_write_access(
                process_id,
                format!("Subscriber[{}/{}].subscribe_to_queue", topic_id, queue_id),
                app.as_ref(),
            )
            .await;

        write_access.data.update_queue_type(queue_type);

        kicked_subscriber_result = write_access.data.subscribers.subscribe(
            subscriber_id,
            topic.clone(),
            topic_queue.clone(),
            session.clone(),
        );

        app.logs
            .add_info(
                Some(topic_id.to_string()),
                crate::app::logs::SystemProcess::QueueOperation,
                format!(
                    "Subscribed. SessionId: {}. SubscriberId: {}",
                    session.id, subscriber_id
                ),
                format!(
                    "Session {} is subscribing to the {}/{} ",
                    session.get_name().await,
                    topic_id,
                    queue_id
                ),
            )
            .await;
    }

    match kicked_subscriber_result {
        Ok(kicke_subscriber) => {
            if let Some(kicked_subscriber) = kicke_subscriber {
                crate::operations::subscriber::handle_subscriber_remove(kicked_subscriber).await;
            }

            super::delivery::deliver_to_queue(
                process_id,
                app.clone(),
                topic.clone(),
                topic_queue.clone(),
            );
        }
        Err(err) => match err {
            SubscribeErrorResult::SubscriberWithIdExists => {
                panic!(
                    "Somehow we generated the same ID {} for the new subscriber {}/{}",
                    subscriber_id, topic_id, queue_id
                );
            }
            SubscribeErrorResult::SubscriberOfSameConnectionExists => {
                panic!(
                        "Somehow we subscribe second time to the same queue {}/{} the same session_id {} for the new subscriber. Most probably there is a bug on the client",
                        topic_id, queue_id, subscriber_id
                    );
            }
        },
    }

    Ok(())
}

pub async fn handle_subscriber_remove(mut subscriber: QueueSubscriber) {
    let messages = subscriber.reset_delivery();

    if let Some(messages_on_delivery) = &messages {
        subscriber
            .queue
            .mark_not_delivered(messages_on_delivery)
            .await;
    }
}
