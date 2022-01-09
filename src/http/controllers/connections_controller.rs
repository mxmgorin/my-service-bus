use async_trait::async_trait;
use std::sync::Arc;

use my_http_server::{
    middlewares::{
        controllers::{actions::DeleteAction, documentation::HttpActionDescription},
        swagger::types::{HttpInputParameter, HttpParameterInputSource, HttpParameterType},
    },
    HttpContext, HttpFailResult, HttpOkResult, WebContentType,
};

use crate::{app::AppContext, sessions::SessionId};

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
    fn get_controller_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            name: "Connections",
            description: "Disconnect connection",
            out_content_type: WebContentType::Json,
        }
        .into()
    }

    fn get_in_parameters_description(&self) -> Option<Vec<HttpInputParameter>> {
        Some(vec![HttpInputParameter {
            name: "id".to_string(),
            param_type: HttpParameterType::String,
            description: "Id of connection".to_string(),
            source: HttpParameterInputSource::Query,
            required: true,
        }])
    }

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
