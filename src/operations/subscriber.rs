use std::sync::Arc;

use my_service_bus_shared::queue::TopicQueueType;

use crate::{
    app::AppContext,
    queue_subscribers::{QueueSubscriber, SubscribeErrorResult},
    queues::TopicQueue,
};

use super::OperationFailResult;

pub async fn subscribe_to_queue(
    app: Arc<AppContext>,
    topic_id: String,
    queue_id: String,
    queue_type: TopicQueueType,
    session_id: i64,
    delivery_packet_version: i32,
) -> Result<(), OperationFailResult> {
    let mut topic = app.topic_list.get(topic_id.as_str()).await;

    if topic.is_none() {
        if app.auto_create_topic_on_subscribe {
            topic = Some(app.topic_list.add_if_not_exists(topic_id).await);
        } else {
            return Err(OperationFailResult::TopicNotFound { topic_id });
        }
    }

    let topic = topic.unwrap();

    let mut topic_data = topic.data.lock().await;

    let topic_queue = topic_data.queues.add_queue_if_not_exists(
        topic.topic_id.to_string(),
        queue_id,
        queue_type.clone(),
    );

    let subscriber_id = app.subscriber_id_generator.get_next_subsriber_id();

    topic_queue.update_queue_type(queue_type);

    let kicked_subscriber_result = topic_queue.subscribers.subscribe(
        subscriber_id,
        topic.topic_id.to_string(),
        topic_queue.queue_id.to_string(),
        session_id,
        delivery_packet_version,
    );

    app.logs.add_info(
        Some(topic.topic_id.to_string()),
        crate::app::logs::SystemProcess::QueueOperation,
        format!(
            "Subscribed. SessionId: {}. SubscriberId: {}",
            session_id, subscriber_id
        ),
        format!(
            "Session {} is subscribing to the {}/{} ",
            session_id, topic.topic_id, topic_queue.queue_id
        ),
    );

    match kicked_subscriber_result {
        Ok(kicke_subscriber) => {
            if let Some(kicked_subscriber) = kicke_subscriber {
                remove_subscriber(topic_queue, kicked_subscriber);
            }

            super::delivery::try_to_deliver(app, &topic, &mut topic_data);

            Ok(())
        }
        Err(err) => match err {
            SubscribeErrorResult::SubscriberWithIdExists => {
                panic!(
                    "Somehow we generated the same ID {} for the new subscriber {}/{}",
                    subscriber_id, topic.topic_id, topic_queue.queue_id
                );
            }
            SubscribeErrorResult::SubscriberOfSameConnectionExists => {
                panic!(
                        "Somehow we subscribe second time to the same queue {}/{} the same session_id {} for the new subscriber. Most probably there is a bug on the client",
                        topic.topic_id, topic_queue.queue_id, subscriber_id
                    );
            }
        },
    }
}

pub fn remove_subscriber(queue: &mut TopicQueue, mut subscriber: QueueSubscriber) {
    let messages = subscriber.reset_delivery();

    if let Some(delivery_bucket) = &messages {
        queue.mark_not_delivered(delivery_bucket);
    }
}
