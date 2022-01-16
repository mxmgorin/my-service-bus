use async_trait::async_trait;
use std::sync::Arc;

use my_http_server::{
    middlewares::controllers::{
        actions::DeleteAction,
        documentation::{data_types::HttpObjectStructure, HttpActionDescription},
    },
    HttpContext, HttpFailResult, HttpOkResult,
};

use super::super::contracts::response;
use crate::{app::AppContext, sessions::SessionId};
pub struct SessionsController {
    app: Arc<AppContext>,
}

impl SessionsController {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait]
impl DeleteAction for SessionsController {
    fn get_additional_types(&self) -> Option<Vec<HttpObjectStructure>> {
        None
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: "Sessions",
            description: "Disconnect and kick session",

            input_params: vec![super::super::contracts::input_parameters::connection_id()].into(),

            results: vec![
                response::empty("Session is kicked"),
                response::session_is_not_found(),
            ],
        }
        .into()
    }

    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let query = ctx.get_query_string()?;

        let id: SessionId = query.get_required_parameter("connectionId")?;

        match self.app.sessions.get(id).await {
            Some(session) => {
                session.disconnect().await;
                Ok(HttpOkResult::Empty)
            }
            None => Err(HttpFailResult::as_not_found(
                format!("Session {} is not found", id),
                false,
            )),
        }
    }
}
