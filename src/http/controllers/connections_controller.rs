use async_trait::async_trait;
use std::sync::Arc;

use my_http_server::{
    middlewares::controllers::{actions::DeleteAction, documentation::HttpActionDescription},
    HttpContext, HttpFailResult, HttpOkResult,
};

use crate::{app::AppContext, sessions::SessionId};

use super::consts::*;

pub struct ConnectionsController {
    app: Arc<AppContext>,
}

impl ConnectionsController {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait]
impl DeleteAction for ConnectionsController {
    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            name: "Connections",
            description: "Disconnect connection",

            input_params: vec![get_connection_id_parameter()].into(),

            results: vec![],
        }
        .into()
    }

    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let query = ctx.get_query_string()?;

        let id: SessionId = query.get_required_parameter("connectionId")?;

        match self.app.sessions.get(id).await {
            Some(session) => {
                session.disconnect().await;

                let result = HttpOkResult::Text {
                    text: "Session is removed".to_string(),
                };
                Ok(result)
            }
            None => Err(HttpFailResult::as_not_found(
                format!("Session {} is not found", id),
                false,
            )),
        }
    }
}
