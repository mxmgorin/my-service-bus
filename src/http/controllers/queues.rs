use my_service_bus_shared::MessageId;

use crate::{
    app::AppContext,
    http::{http_ctx::HttpContext, HttpFailResult, HttpOkResult},
};

pub async fn set_message_id(
    app: &AppContext,
    ctx: HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string();
    let topic_id = query.get_query_required_string_parameter("topicId")?;
    let queue_id = query.get_query_required_string_parameter("queueId")?;
    let message_id: MessageId = query.get_query_required_parameter("messageId")?;

    crate::operations::queues::set_message_id(app, topic_id, queue_id, message_id).await?;

    Ok(HttpOkResult::Ok)
}

pub async fn delete(app: &AppContext, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string();

    let topic_id = query.get_query_required_string_parameter("topicId")?;
    let queue_id = query.get_query_required_string_parameter("queueId")?;

    crate::operations::queues::delete_queue(app, topic_id, queue_id).await?;

    Ok(HttpOkResult::Ok)
}

pub async fn get_queues(
    app: &AppContext,
    ctx: HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string();
    let topic_id = query.get_query_required_string_parameter("topicId")?;

    let topic = app.topic_list.get(topic_id).await;

    if topic.is_none() {
        return Err(HttpFailResult::as_not_found(format!(
            "Topic {} not found",
            topic_id
        )));
    }

    let topic = topic.unwrap();

    let mut result = Vec::new();

    {
        let topic_data = topic.data.lock().await;
        for queue in topic_data.queues.get_queues() {
            result.push(queue.queue_id.clone());
        }
    }

    return HttpOkResult::create_json_response(result);
}
