use crate::http::controllers::extensions::HttpContextExtensions;
use std::sync::Arc;

use async_trait::async_trait;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use my_http_server_controllers::controllers::{
    actions::PostAction, documentation::HttpActionDescription,
};

use crate::app::AppContext;

use super::models::PingInputModel;
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
    fn get_route(&self) -> &str {
        "/Greeting/Ping"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: "Greeting",
            description: "Ping Http Session",
            input_params: PingInputModel::get_input_params().into(),
            results: super::super::contracts::response::empty_and_authorized("Ping is done Ok"),
        }
        .into()
    }

    async fn handle_request(&self, ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let input_data = PingInputModel::parse_http_input(ctx).await?;

        let http_session = self
            .app
            .get_http_session(input_data.authorization.as_str())
            .await?;

        http_session.as_ref().connection.unwrap_as_http().ping();

        HttpOutput::Empty.into_ok_result(true).into()
    }
}
