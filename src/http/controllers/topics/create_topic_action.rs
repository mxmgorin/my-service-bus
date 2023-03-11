use std::sync::Arc;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use crate::app::AppContext;
use crate::http::controllers::topics::models::CreateTopicRequestContract;

#[my_http_server_swagger::http_route(
method: "POST",
route: "/Topics/Create",
description: "Create topic",
controller: "Topics",
input_data: "CreateTopicRequestContract",
summary: "",
result: [
{status_code: 200, description: "Ok response"},
]
)]
pub struct CreateTopicAction {
    app: Arc<AppContext>,
}

impl CreateTopicAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &CreateTopicAction,
    input_data: CreateTopicRequestContract,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    crate::operations::publisher::create_topic_if_not_exists(
        &action.app,
        None,
        input_data.topic_id.as_ref(),
    )
        .await?;

    HttpOutput::as_text("Topic is created".to_string())
        .into_ok_result(true)
        .into()
}