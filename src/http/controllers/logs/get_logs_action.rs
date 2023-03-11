use std::sync::Arc;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};
use rust_extensions::StopWatch;
use crate::app::AppContext;

#[my_http_server_swagger::http_route(
method: "GET",
route: "/Logs",
description: "Show Logs",
controller: "Logs",
summary: "",
result: [
{status_code: 200, description: "Ok response"},
]
)]
pub struct GetLogsAction {
    app: Arc<AppContext>,
}

impl GetLogsAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &GetLogsAction,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let mut sw = StopWatch::new();
    sw.start();
    let logs = action.app.logs.get().await;

    return super::renderers::compile_result("logs", logs, sw);
}