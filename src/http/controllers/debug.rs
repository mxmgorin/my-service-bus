use rust_extensions::StringBuilder;

use crate::{
    app::AppContext,
    http::{http_ctx::HttpContext, HttpFailResult, HttpOkResult},
};

pub async fn enable(app: &AppContext, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
    let query_string = ctx.get_query_string();

    let topic_id = query_string.get_query_required_string_parameter("topic")?;
    let queue_id = query_string.get_query_required_string_parameter("queue")?;

    app.set_debug_topic_and_queue(topic_id, queue_id).await;

    Ok(HttpOkResult::Text {
        text: "Ok".to_string(),
    })
}

pub async fn disable(app: &AppContext) -> Result<HttpOkResult, HttpFailResult> {
    app.disable_debug_topic_and_queue().await;

    Ok(HttpOkResult::Text {
        text: "Ok".to_string(),
    })
}

pub async fn get_on_delivery(
    app: &AppContext,
    ctx: HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let query_string = ctx.get_query_string();

    let topic_id = query_string.get_query_required_string_parameter("topic")?;
    let queue_id = query_string.get_query_required_string_parameter("queue")?;
    let subscriber_id = query_string.get_query_required_parameter::<i64>("subscriberid")?;

    let topic = app.topic_list.get(topic_id).await;
    if topic.is_none() {
        return Err(HttpFailResult::as_not_found("Topic not found".to_string()));
    }

    let topic = topic.unwrap();

    let ids = {
        let topic_data = topic.get_access("debug.get_on_delivery").await;

        let queue = topic_data.queues.get(queue_id);

        if queue.is_none() {
            return Err(HttpFailResult::as_not_found("Queue not found".to_string()));
        }

        let queue = queue.unwrap();

        queue.get_messages_on_delivery(subscriber_id)
    };

    return Ok(HttpOkResult::Text {
        text: format!("{:?}", ids),
    });
}

pub async fn locks(app: &AppContext) -> Result<HttpOkResult, HttpFailResult> {
    let topics = app.topic_list.get_all().await;

    let mut result = StringBuilder::new();

    result.append_line("Locks:");

    for topic in topics {
        if let Some(lines) = topic.get_locks().await {
            result.append_line("");
            result.append_line(format!("{}", topic.topic_id).as_str());

            for line in lines {
                result.append_str(line.as_str());
            }
        }
    }

    return Ok(HttpOkResult::Text {
        text: format!("{}", result.to_string_utf8().unwrap()),
    });
}
