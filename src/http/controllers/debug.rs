use my_service_bus_shared::debug::LockItem;
use rust_extensions::{date_time::DateTimeAsMicroseconds, StringBuilder};

use crate::{
    app::AppContext,
    http::{http_ctx::HttpContext, HttpFailResult, HttpOkResult},
};

pub async fn get(app: &AppContext) -> Result<HttpOkResult, HttpFailResult> {
    let logs = app.locks.get_locks().await;

    let text = compile_result(&logs);

    Ok(HttpOkResult::Text { text })
}

fn compile_result(items: &[LockItem]) -> String {
    let mut result = StringBuilder::new();

    for itm in items {
        let date = DateTimeAsMicroseconds::new(itm.date);
        result.append_line(
            format!("{} {} [{}]", date.to_rfc3339(), itm.to_string(), itm.id,).as_str(),
        );
    }

    result.to_string_utf8().unwrap()
}

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
    let subscriber_id = query_string.get_query_required_parameter::<i64>("queue")?;

    let topic = app.topic_list.get(topic_id).await;
    if topic.is_none() {
        return Err(HttpFailResult::as_not_found("Topic not found".to_string()));
    }

    let topic = topic.unwrap();

    let queue = topic.get_queue(queue_id).await;

    if queue.is_none() {
        return Err(HttpFailResult::as_not_found("Queue not found".to_string()));
    }

    let queue = queue.unwrap();

    let ids = queue.get_messages_on_delivery(subscriber_id).await;

    return Ok(HttpOkResult::Text {
        text: format!("{:?}", ids),
    });
}
