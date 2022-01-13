use crate::http::controllers::{consts::*, extensions::HttpContextExtensions};
use std::sync::Arc;

use async_trait::async_trait;
use my_http_server::{
    middlewares::controllers::{actions::PostAction, documentation::HttpActionDescription},
    HttpContext, HttpFailResult, HttpOkResult, WebContentType,
};

use crate::app::AppContext;
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
    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            name: "Greeting",
            description: "Ping Http Session",
            out_content_type: WebContentType::Json,
            input_params: Some(vec![get_auth_header_description()]),
        }
        .into()
    }

    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let session_id = ctx.get_required_header(AUTH_HEADER_NAME)?;

        let http_session = self.app.get_http_session(session_id).await?;

        http_session.as_ref().connection.unwrap_as_http().ping();

        Ok(HttpOkResult::Ok)
    }
}
