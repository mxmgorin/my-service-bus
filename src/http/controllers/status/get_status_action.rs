use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use std::sync::Arc;
use crate::app::AppContext;

#[my_http_server_swagger::http_route(
method: "GET",
route: "/Status",
description: "Get status of application",
controller: "Status",
summary: "",
result: [
{status_code: 200, description: "Ok response"},
]
)]
pub struct GetStatusAction {
    app: Arc<AppContext>,
}

impl GetStatusAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &GetStatusAction,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let result = super::index_models::StatusJsonResult::new(action.app.as_ref()).await;

    HttpOutput::as_json(result).into_ok_result(true).into()
}
