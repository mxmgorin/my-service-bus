use std::sync::Arc;

use my_service_bus_shared::{
    date_time::DateTimeAsMicroseconds, queue::TopicQueueType,
    queue_with_intervals::QueueWithIntervals,
};

use crate::{
    app::{AppContext, TEST_QUEUE},
    queues::QueueData,
    sessions::MyServiceBusSession,
    subscribers::SubscriberId,
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
    let mut to_send = Vec::new();

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

    let subscriber_id = app.subscriber_id_generator.get_next_subsriber_id();
    {
        app.enter_lock(
            process_id,
            format!("Subscriber[{}/{}].subscribe_to_queue", topic_id, queue_id),
        )
        .await;
        let mut write_access = topic_queue.data.write().await;

        write_access.queue_type = queue_type;

        write_access
            .subscribers
            .subscribe(subscriber_id, queue_id, the_session.clone());

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

        if matches!(
            write_access.queue_type,
            TopicQueueType::PermanentWithSingleConnection
        ) {
            let subscribers_to_unsubscribe = write_access
                .subscribers
                .get_all_except_this_one(subscriber_id);

            for subscriber_to_unsubscribe_id in subscribers_to_unsubscribe {
                let result = unsubscribe(
                    process_id,
                    session,
                    &mut write_access,
                    subscriber_to_unsubscribe_id,
                )
                .await;

                if let Err(err) = result {
                    app.logs
                        .add_error(
                            None,
                            crate::app::logs::SystemProcess::TcpSocket,
                            "subscriber::subscribe_to_queue".to_string(),
                            format!("Faild to unscrubscribe {}", subscriber_to_unsubscribe_id),
                            Some(format!("{:?}", err)),
                        )
                        .await;
                }
            }
        }

        let result = super::delivery::try_to_complie_next_messages_from_the_queue(
            process_id,
            app.as_ref(),
            topic.as_ref(),
            &mut write_access,
        )
        .await?;

        to_send.extend(result);
        app.exit_lock(process_id).await;
    }

    session
        .add_subscriber(process_id, subscriber_id, topic_id, queue_id)
        .await?;

    //Thread safety - we are doing it beyond scope of the queue lock;
    for (tcp_contract, session, subscriber_id) in to_send {
        session
            .send_and_set_on_delivery(process_id, tcp_contract, subscriber_id)
            .await;
    }

    Ok(())
}

pub async fn unsubscribe(
    process_id: i64,
    session: &MyServiceBusSession,
    queue: &mut QueueData,
    subscriber_id: SubscriberId,
) -> Result<(), OperationFailResult> {
    let removed_subscriber = queue.subscribers.remove(&subscriber_id);

    if removed_subscriber.is_none() {
        return Err(OperationFailResult::SubscriberNotFound { id: subscriber_id });
    }

    let removed_subscriber = removed_subscriber.unwrap();

    session.remove_subscriber(process_id, subscriber_id).await;

    if queue.queue_id == TEST_QUEUE {
        if let Some(message_bucket) = removed_subscriber.messages_on_delivery {
            queue.confirmed_non_delivered(&message_bucket)
        }
    }

    queue.last_ubsubscribe = DateTimeAsMicroseconds::now();

    Ok(())
}

pub async fn confirm_delivery(
    process_id: i64,
    app: Arc<AppContext>,
    topic_id: &str,
    queue_id: &str,
    session: &MyServiceBusSession,
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

    let start_time: DateTimeAsMicroseconds;

    let mut delivered_messages_amount: Option<usize> = None;

    let mut to_send = Vec::new();

    {
        let mut write_access = topic_queue.data.write().await;

        let subscriber = write_access
            .subscribers
            .get_by_id_mut(subscriber_id)
            .ok_or(OperationFailResult::SubscriberNotFound { id: subscriber_id })?;

        let messages_on_delivery = subscriber.reset();

        start_time = subscriber.start_delivering;

        if let Some(messages_on_delivery) = messages_on_delivery {
            delivered_messages_amount = Some(messages_on_delivery.messages_count());
            write_access.confirmed_delivered(messages_on_delivery);
        }

        let result = super::delivery::try_to_complie_next_messages_from_the_queue(
            process_id,
            app.as_ref(),
            topic.as_ref(),
            &mut write_access,
        )
        .await;

        match result {
            Ok(msg) => {
                to_send.extend(msg);
            }
            Err(err) => {
                app.logs
                    .add_error(
                        Some(topic.topic_id.to_string()),
                        crate::app::logs::SystemProcess::TcpSocket,
                        "subscribers::confirm_delivery".to_string(),
                        format!(
                            "Faild to deliver next data to subscriber {}. Queue {}",
                            subscriber_id, queue_id
                        ),
                        Some(format!("{:?}", err)),
                    )
                    .await;
            }
        }
    }

    if let Some(delivered_messages) = delivered_messages_amount {
        let dur = DateTimeAsMicroseconds::now().duration_since(start_time);
        session
            .set_delivered_statistic(
                process_id,
                subscriber_id,
                delivered_messages as usize,
                dur.as_micros() as usize,
            )
            .await;
    }

    //Thread safety - we are doing it beyond scope of the queue lock;
    for (tcp_contract, session, subscriber_id) in to_send {
        session
            .send_and_set_on_delivery(process_id, tcp_contract, subscriber_id)
            .await;
    }

    Ok(())
}

pub async fn confirm_non_delivery(
    process_id: i64,
    app: Arc<AppContext>,
    topic_id: &str,
    queue_id: &str,
    session: &MyServiceBusSession,
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

    let start_time: DateTimeAsMicroseconds;

    let mut delivered_messages_amount: Option<usize> = None;
    let mut to_send = Vec::new();
    {
        let mut write_access = topic_queue.data.write().await;

        let subscriber = write_access
            .subscribers
            .get_by_id_mut(subscriber_id)
            .ok_or(OperationFailResult::SubscriberNotFound { id: subscriber_id })?;

        let messages_on_delivery = subscriber.reset();

        start_time = subscriber.start_delivering;

        if let Some(messages_on_delivery) = &messages_on_delivery {
            delivered_messages_amount = Some(messages_on_delivery.messages_count());
            write_access.confirmed_non_delivered(messages_on_delivery);
        }

        let result = super::delivery::try_to_complie_next_messages_from_the_queue(
            process_id,
            app.as_ref(),
            topic.as_ref(),
            &mut write_access,
        )
        .await;

        match result {
            Ok(msg) => {
                to_send.extend(msg);
            }
            Err(err) => {
                app.logs
                    .add_error(
                        Some(topic.topic_id.to_string()),
                        crate::app::logs::SystemProcess::TcpSocket,
                        "subscribers::confirm_non_delivery".to_string(),
                        format!(
                            "Faild to deliver next data to subscriber {}. Queue {}",
                            subscriber_id, queue_id
                        ),
                        Some(format!("{:?}", err)),
                    )
                    .await;
            }
        }
    }

    if let Some(delivered_messages) = delivered_messages_amount {
        session
            .set_not_delivered_statistic(
                process_id,
                subscriber_id,
                delivered_messages as i32,
                DateTimeAsMicroseconds::now()
                    .duration_since(start_time)
                    .as_micros() as i32,
            )
            .await;
    }

    //Thread safety - we are doing it beyond scope of the queue lock;
    for (tcp_contract, session, subscriber_id) in to_send {
        session
            .send_and_set_on_delivery(process_id, tcp_contract, subscriber_id)
            .await;
    }

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
    let mut to_send = Vec::new();

    {
        let topic =
            app.topic_list
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

        let subscriber = write_access
            .subscribers
            .get_by_id_mut(subscriber_id)
            .ok_or(OperationFailResult::SubscriberNotFound { id: subscriber_id })?;

        let messages_on_delivery = subscriber.reset();

        if let Some(messages_on_delivery) = messages_on_delivery {
            write_access.confirmed_some_delivered(messages_on_delivery, confirmed_messages)?;
        }

        let result = super::delivery::try_to_complie_next_messages_from_the_queue(
            process_id,
            app.as_ref(),
            topic.as_ref(),
            &mut write_access,
        )
        .await;

        match result {
            Ok(msg) => {
                to_send.extend(msg);
            }
            Err(err) => {
                app.logs
                    .add_error(
                        Some(topic.topic_id.to_string()),
                        crate::app::logs::SystemProcess::TcpSocket,
                        "subscribers::some_messages_are_not_confirmed".to_string(),
                        format!(
                            "Faild to deliver next data to subscriber {}. Queue {}",
                            subscriber_id, queue_id
                        ),
                        Some(format!("{:?}", err)),
                    )
                    .await;
            }
        }
    }

    //Thread safety - we are doing it beyond scope of the queue lock;
    for (tcp_contract, session, subscriber_id) in to_send {
        session
            .send_and_set_on_delivery(process_id, tcp_contract, subscriber_id)
            .await;
    }

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
    let mut to_send = Vec::new();

    {
        let topic =
            app.topic_list
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

        let subscriber = write_access
            .subscribers
            .get_by_id_mut(subscriber_id)
            .ok_or(OperationFailResult::SubscriberNotFound { id: subscriber_id })?;

        let messages_on_delivery = subscriber.reset();

        if let Some(messages_on_delivery) = messages_on_delivery {
            write_access
                .confirmed_some_not_delivered(messages_on_delivery, not_confirmed_messages)?;
        }

        let result = super::delivery::try_to_complie_next_messages_from_the_queue(
            process_id,
            app.as_ref(),
            topic.as_ref(),
            &mut write_access,
        )
        .await;

        match result {
            Ok(msg) => {
                to_send.extend(msg);
            }
            Err(err) => {
                app.logs
                    .add_error(
                        Some(topic.topic_id.to_string()),
                        crate::app::logs::SystemProcess::TcpSocket,
                        "subscribers::some_messages_are_not_confirmed".to_string(),
                        format!(
                            "Faild to deliver next data to subscriber {}. Queue {}",
                            subscriber_id, queue_id
                        ),
                        Some(format!("{:?}", err)),
                    )
                    .await;
            }
        }
    }

    //Thread safety - we are doing it beyond scope of the queue lock;
    for (tcp_contract, session, subscriber_id) in to_send {
        session
            .send_and_set_on_delivery(process_id, tcp_contract, subscriber_id)
            .await;
    }

    Ok(())
}
