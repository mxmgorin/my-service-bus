use super::models::EnableDebugInputModel;
use crate::app::AppContext;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use std::sync::Arc;

#[my_http_server_swagger::http_route(
    method: "POST",
    route: "/Debug/Enable",
    description: "Enable debug mode for specific queue",
    controller: "Debug",
    input_data: "EnableDebugInputModel",
    summary: "",
    result: [
        {status_code: 200, description: "Ok response"},
    ]
)]
pub struct EnableDebugModeAction {
    app: Arc<AppContext>,
}

impl EnableDebugModeAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &EnableDebugModeAction,
    request: EnableDebugInputModel,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    action
        .app
        .set_debug_topic_and_queue(request.topic_id.as_ref(), request.queue_id.as_ref())
        .await;

    HttpOutput::Empty.into_ok_result(true).into()
}
