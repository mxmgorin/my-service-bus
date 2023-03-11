use crate::app::AppContext;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use std::sync::Arc;
use crate::http::controllers::sessions::DeleteSessionInputContract;

#[my_http_server_swagger::http_route(
method: "DELETE",
route: "/Sessions",
description: "Disconnect and kicks a session",
controller: "Sessions",
input_data: "DeleteSessionInputContract",
summary: "",
result: [
{status_code: 200, description: "Ok response"},
]
)]
pub struct DeleteSessionAction {
    app: Arc<AppContext>,
}

impl DeleteSessionAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &DeleteSessionAction,
    input_data: DeleteSessionInputContract,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    match action.app.sessions.get(input_data.connection_id).await {
        Some(session) => {
            session.disconnect().await;
            HttpOutput::Empty.into_ok_result(true).into()
        }
        None => Err(HttpFailResult::as_not_found(
            format!("Session {} is not found", input_data.connection_id),
            false,
        )),
    }
}
