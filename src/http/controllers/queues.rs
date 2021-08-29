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
