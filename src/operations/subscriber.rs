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
    session: &MyServiceBusSession,
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

    let the_session = app.as_ref().sessions.get_by_id(session.id).await;

    if the_session.is_none() {
        app.logs
            .add_error(
                Some(topic_id.to_string()),
                crate::app::logs::SystemProcess::QueueOperation,
                format!("subscribe_to_queue {}", queue_id),
                format!("Somehow subscriber {} is not found anymore", session.id),
                None,
            )
            .await;
    }

    let the_session = the_session.unwrap();

    let kicked_subscriber_result;

    let subscriber_id = app.subscriber_id_generator.get_next_subsriber_id();
    {
        app.enter_lock(
            process_id,
            format!("Subscriber[{}/{}].subscribe_to_queue", topic_id, queue_id),
        )
        .await;
        let mut write_access = topic_queue.data.write().await;

        write_access.update_queue_type(queue_type);

        kicked_subscriber_result = write_access.subscribers.subscribe(
            subscriber_id,
            session.id,
            topic.clone(),
            topic_queue.clone(),
            the_session.clone(),
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
                    session.get_name(process_id,).await,
                    topic_id,
                    queue_id
                ),
            )
            .await;

        app.exit_lock(process_id).await;
    }

    if let Some(kicked_subscriber) = kicked_subscriber_result {
        crate::operations::subscriber::handle_subscriber_remove(
            process_id,
            app.as_ref(),
            kicked_subscriber,
        )
        .await;
    }

    super::delivery::deliver_to_queue(process_id, app.clone(), topic.clone(), topic_queue.clone());

    Ok(())
}

pub async fn handle_subscriber_remove(
    process_id: i64,
    app: &AppContext,
    mut subscriber: QueueSubscriber,
) {
    let messages = subscriber.reset_delivery();

    if let Some(messages_on_delivery) = &messages {
        app.enter_lock(
            process_id,
            format!(
                "handle_subscriber_remove[{}/{}]",
                subscriber.queue.topic_id, subscriber.queue.queue_id
            ),
        )
        .await;
        let mut write_access = subscriber.queue.data.write().await;
        write_access.mark_not_delivered(messages_on_delivery);
        app.exit_lock(process_id).await;
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

    let mut write_access = topic_queue.data.write().await;

    if let Err(err) = write_access.confirmed_delivered(subscriber_id) {
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

    let mut write_access = topic_queue.data.write().await;
    if let Err(err) = write_access.confirmed_non_delivered(subscriber_id) {
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

    let mut write_access = topic_queue.data.write().await;
    if let Err(err) = write_access.confirmed_some_delivered(subscriber_id, confirmed_messages) {
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

    let mut write_access = topic_queue.data.write().await;
    if let Err(err) =
        write_access.confirmed_some_not_delivered(subscriber_id, not_confirmed_messages)
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
