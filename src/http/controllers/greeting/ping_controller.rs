use std::sync::Arc;

use async_trait::async_trait;
use my_http_utils::{HttpContext, HttpFailResult, HttpOkResult, WebContentType};

use crate::{
    app::AppContext,
    http::middlewares::{
        controllers::{actions::PostAction, documentation::HttpActionDescription},
        swagger::types::{
            SwaggerInputParameter, SwaggerParameterInputSource, SwaggerParameterType,
        },
    },
    sessions::SessionConnection,
};
pub struct PingController {
    app: Arc<AppContext>,
}

impl PingController {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait]
impl PostAction for PingController {
    fn get_controller_description(&self) -> HttpActionDescription {
        HttpActionDescription {
            name: "Greeting",
            description: "Ping Http Session",
            out_content_type: WebContentType::Json,
        }
    }

    fn get_in_parameters_description(&self) -> Option<Vec<SwaggerInputParameter>> {
        Some(vec![SwaggerInputParameter {
            name: "SESSION".to_string(),
            param_type: SwaggerParameterType::String,
            description: "Session, issued by greeting method".to_string(),
            source: SwaggerParameterInputSource::Headers,
            required: true,
        }])
    }

    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let session_id = ctx.get_required_header("SESSION")?;

        match self.app.sessions.get_http(session_id).await {
            Some(session) => {
                if let SessionConnection::Http(http_data) = &session.connection {
                    http_data.ping();
                    Ok(HttpOkResult::Ok)
                } else {
                    Err(HttpFailResult::as_unauthorized(Some(
                        "Session should has HTTP Type".to_string(),
                    )))
                }
            }
            None => Err(HttpFailResult::as_unauthorized(None)),
        }
    }
}
