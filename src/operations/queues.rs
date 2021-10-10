use my_service_bus_shared::queue_with_intervals::QueueIndexRange;

use crate::app::AppContext;

use super::OperationFailResult;

pub async fn set_message_id(
    app: &AppContext,
    topic_id: &str,
    queue_id: &str,
    message_id: i64,
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

    let mut topic_queue_data = topic_queue.data.write().await;

    let max_message_id = topic.get_message_id().await;

    let mut intervals = Vec::new();

    intervals.push(QueueIndexRange {
        from_id: message_id,
        to_id: max_message_id,
    });

    topic_queue_data.queue.reset(intervals);

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
