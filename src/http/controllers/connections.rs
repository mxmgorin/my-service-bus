use async_trait::async_trait;
use std::sync::Arc;

use my_http_utils::{HttpContext, HttpFailResult, HttpOkResult};

use crate::{
    app::AppContext, http::middlewares::controllers::actions::DeleteAction, sessions::SessionId,
};

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
    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let query = ctx.get_query_string()?;

        let id: SessionId = query.get_required_parameter("id")?;

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
