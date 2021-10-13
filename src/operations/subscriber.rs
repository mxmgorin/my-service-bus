use std::sync::Arc;

use my_service_bus_shared::{queue::TopicQueueType, queue_with_intervals::QueueWithIntervals};

use crate::{
    app::AppContext,
    queue_subscribers::{QueueSubscriber, SubscriberId},
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
    let topic = app
        .topic_list
        .get(topic_id)
        .await
        .ok_or(OperationFailResult::TopicNotFound {
            topic_id: topic_id.to_string(),
        })?;

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

    if let Some(kicked_subscriber) = kicked_subscriber_result {
        crate::operations::subscriber::handle_subscriber_remove(kicked_subscriber).await;
    }

    super::delivery::deliver_to_queue(process_id, app.clone(), topic.clone(), topic_queue.clone());

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

pub async fn confirm_delivery(
    process_id: i64,
    app: Arc<AppContext>,
    topic_id: &str,
    queue_id: &str,
    subscriber_id: SubscriberId,
) -> Result<(), OperationFailResult> {
    let topic = app
        .topic_list
        .get(topic_id)
        .await
        .ok_or(OperationFailResult::TopicNotFound {
            topic_id: topic_id.to_string(),
        })?;

    let topic_queue =
        topic
            .get_queue(queue_id)
            .await
            .ok_or(OperationFailResult::QueueNotFound {
                queue_id: queue_id.to_string(),
            })?;

    if let Err(err) = topic_queue.confirmed_delivered(subscriber_id).await {
        app.logs
            .add_fatal_error(
                crate::app::logs::SystemProcess::DeliveryOperation,
                "confirm_delivery".to_string(),
                format!("{:?}", err),
            )
            .await
    }

    super::delivery::deliver_to_queue(process_id, app.clone(), topic.clone(), topic_queue.clone());

    Ok(())
}

pub async fn confirm_non_delivery(
    process_id: i64,
    app: Arc<AppContext>,
    topic_id: &str,
    queue_id: &str,
    subscriber_id: SubscriberId,
) -> Result<(), OperationFailResult> {
    let topic = app
        .topic_list
        .get(topic_id)
        .await
        .ok_or(OperationFailResult::TopicNotFound {
            topic_id: topic_id.to_string(),
        })?;

    let topic_queue =
        topic
            .get_queue(queue_id)
            .await
            .ok_or(OperationFailResult::QueueNotFound {
                queue_id: queue_id.to_string(),
            })?;

    if let Err(err) = topic_queue.confirmed_non_delivered(subscriber_id).await {
        app.logs
            .add_fatal_error(
                crate::app::logs::SystemProcess::DeliveryOperation,
                "confirm_non_delivery".to_string(),
                format!("{:?}", err),
            )
            .await
    }

    super::delivery::deliver_to_queue(process_id, app.clone(), topic.clone(), topic_queue.clone());

    Ok(())
}

//TODO - Plug partialy metrics
pub async fn some_messages_are_confirmed(
    process_id: i64,
    app: Arc<AppContext>,
    topic_id: &str,
    queue_id: &str,
    subscriber_id: SubscriberId,
    confirmed_messages: QueueWithIntervals,
) -> Result<(), OperationFailResult> {
    let topic = app
        .topic_list
        .get(topic_id)
        .await
        .ok_or(OperationFailResult::TopicNotFound {
            topic_id: topic_id.to_string(),
        })?;

    let topic_queue =
        topic
            .get_queue(queue_id)
            .await
            .ok_or(OperationFailResult::QueueNotFound {
                queue_id: queue_id.to_string(),
            })?;

    if let Err(err) = topic_queue
        .confirmed_some_delivered(subscriber_id, confirmed_messages)
        .await
    {
        app.logs
            .add_fatal_error(
                crate::app::logs::SystemProcess::DeliveryOperation,
                "some_messages_are_confirmed".to_string(),
                format!("{:?}", err),
            )
            .await
    }

    super::delivery::deliver_to_queue(process_id, app.clone(), topic.clone(), topic_queue.clone());

    Ok(())
}

//TODO - Plug partialy metrics
pub async fn some_messages_are_not_confirmed(
    process_id: i64,
    app: Arc<AppContext>,
    topic_id: &str,
    queue_id: &str,
    subscriber_id: SubscriberId,
    not_confirmed_messages: QueueWithIntervals,
) -> Result<(), OperationFailResult> {
    let topic = app
        .topic_list
        .get(topic_id)
        .await
        .ok_or(OperationFailResult::TopicNotFound {
            topic_id: topic_id.to_string(),
        })?;

    let topic_queue =
        topic
            .get_queue(queue_id)
            .await
            .ok_or(OperationFailResult::QueueNotFound {
                queue_id: queue_id.to_string(),
            })?;

    if let Err(err) = topic_queue
        .confirmed_some_not_delivered(subscriber_id, not_confirmed_messages)
        .await
    {
        app.logs
            .add_fatal_error(
                crate::app::logs::SystemProcess::DeliveryOperation,
                "some_messages_are_not_confirmed".to_string(),
                format!("{:?}", err),
            )
            .await
    }

    super::delivery::deliver_to_queue(process_id, app.clone(), topic.clone(), topic_queue.clone());

    Ok(())
}
