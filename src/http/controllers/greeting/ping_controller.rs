use crate::http::controllers::{consts::*, extensions::HttpContextExtensions};
use std::sync::Arc;

use async_trait::async_trait;
use my_http_server::{
    middlewares::controllers::{
        actions::PostAction,
        documentation::{data_types::HttpObjectType, HttpActionDescription},
    },
    HttpContext, HttpFailResult, HttpOkResult,
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
    fn get_additional_types(&self) -> Option<Vec<HttpObjectType>> {
        None
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            name: "Greeting",
            description: "Ping Http Session",
            input_params: Some(vec![get_auth_header_description()]),
            results: super::super::consts::get_empty_result(),
        }
        .into()
    }

    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let session_id = ctx.get_required_header(AUTH_HEADER_NAME)?;

        let http_session = self.app.get_http_session(session_id).await?;

        http_session.as_ref().connection.unwrap_as_http().ping();

        Ok(HttpOkResult::Empty)
    }
}
