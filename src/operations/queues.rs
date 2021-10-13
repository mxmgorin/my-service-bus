use my_service_bus_shared::MessageId;

use crate::app::AppContext;

use super::OperationFailResult;

pub async fn set_message_id(
    app: &AppContext,
    topic_id: &str,
    queue_id: &str,
    message_id: MessageId,
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

    let max_message_id = topic.get_message_id().await;
    topic_queue.set_message_id(message_id, max_message_id).await;

    Ok(())
}

pub async fn delete_queue(
    app: &AppContext,
    topic_id: &str,
    queue_id: &str,
) -> Result<(), OperationFailResult> {
    let topic = app
        .topic_list
        .get(topic_id)
        .await
        .ok_or(OperationFailResult::TopicNotFound {
            topic_id: topic_id.to_string(),
        })?;

    topic.queues.delete_queue(queue_id).await;

    Ok(())
}
