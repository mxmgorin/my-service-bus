use std::sync::Arc;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use crate::app::AppContext;
use super::models::{JsonTopicResult, JsonTopicsResult};

#[my_http_server_swagger::http_route(
method: "GET",
route: "/Topics",
description: "Get list of topics",
controller: "Topics",
summary: "",
result: [
{status_code: 200, description: "Ok response"},
]
)]
pub struct GetTopicsAction {
    app: Arc<AppContext>,
}

impl GetTopicsAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &GetTopicsAction,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let topics = action.app.topic_list.get_all().await;
    let mut items: Vec<JsonTopicResult> = Vec::new();

    for topic in topics {
        let item = JsonTopicResult::new(topic.as_ref()).await;

        items.push(item);
    }

    let contract = JsonTopicsResult { items };

    HttpOutput::as_json(contract).into_ok_result(true).into()
}
