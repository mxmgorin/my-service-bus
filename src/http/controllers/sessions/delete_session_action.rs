use async_trait::async_trait;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use my_http_server_controllers::controllers::{
    actions::DeleteAction, documentation::HttpActionDescription,
};
use std::sync::Arc;

use super::{super::contracts::response, *};
use crate::app::AppContext;
pub struct DeleteSessionAction {
    app: Arc<AppContext>,
}

impl DeleteSessionAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait]
impl DeleteAction for DeleteSessionAction {
    fn get_route(&self) -> &str {
        "/Sessions"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: "Sessions",
            description: "Disconnect and kick session",

            input_params: DeleteSessionInputContract::get_input_params().into(),

            results: vec![
                response::empty("Session is kicked"),
                response::session_is_not_found(),
            ],
        }
        .into()
    }

    async fn handle_request(&self, ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let input_data = DeleteSessionInputContract::parse_http_input(ctx).await?;

        match self.app.sessions.get(input_data.connection_id).await {
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
}
