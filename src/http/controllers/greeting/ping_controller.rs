use crate::http::controllers::extensions::HttpContextExtensions;
use std::sync::Arc;

use async_trait::async_trait;
use my_http_server::{
    middlewares::controllers::{
        actions::PostAction,
        documentation::{data_types::HttpObjectStructure, HttpActionDescription},
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
    fn get_additional_types(&self) -> Option<Vec<HttpObjectStructure>> {
        None
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: "Greeting",
            description: "Ping Http Session",
            input_params: Some(vec![
                super::super::contracts::input_parameters::auth_header(),
            ]),
            results: super::super::contracts::response::empty_and_authorized("Ping is done Ok"),
        }
        .into()
    }

    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let session_id =
            ctx.get_required_header(super::super::contracts::input_parameters::AUTH_HEADER_NAME)?;

        let http_session = self.app.get_http_session(session_id).await?;

        http_session.as_ref().connection.unwrap_as_http().ping();

        Ok(HttpOkResult::Empty)
    }
}
