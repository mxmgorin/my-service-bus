use std::sync::Arc;

use crate::{http::http_ctx::HttpContext, topics::Topic};

use serde::{Deserialize, Serialize};

use crate::{
    app::AppContext,
    http::{HttpFailResult, HttpOkResult},
};

pub async fn get(app: &AppContext) -> Result<HttpOkResult, HttpFailResult> {
    let topics = app.topic_list.get_all().await;

    let mut response: Vec<JsonTopicResult> = Vec::new();

    for topic in topics {
        let item = JsonTopicResult::new(topic.as_ref()).await;

        response.push(item);
    }

    let root = JsonTopicsResult { items: response };

    HttpOkResult::create_json_response(root)
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(transparent)]
struct JsonTopicsResult {
    items: Vec<JsonTopicResult>,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonTopicResult {
    pub id: String,
    #[serde(rename = "messageId")]
    pub message_id: i64,
}

impl JsonTopicResult {
    pub async fn new(topic: &Topic) -> Self {
        Self {
            id: topic.topic_id.to_string(),
            message_id: topic.get_message_id().await,
        }
    }
}

pub async fn create(
    app: Arc<AppContext>,
    ctx: HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let form_data = ctx.get_form_data().await;
    let topics_id = form_data.get_query_required_string_parameter("topicId")?;

    let process_id = app.process_id_generator.get_process_id().await;

    crate::operations::publisher::create_topic_if_not_exists(process_id, app, None, topics_id)
        .await;

    let result = HttpOkResult::Text {
        text: "Topic is created".to_string(),
    };

    Ok(result)
}
