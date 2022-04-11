use async_trait::async_trait;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use my_http_server_controllers::controllers::{
    actions::GetAction, documentation::HttpActionDescription,
};
use std::sync::Arc;

use crate::app::AppContext;

pub struct StatusController {
    app: Arc<AppContext>,
}

impl StatusController {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait]
impl GetAction for StatusController {
    fn get_route(&self) -> &str {
        "/Status"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: "Status",
            description: "Get status of application",
            input_params: None,
            results: Vec::new(),
        }
        .into()
    }

    async fn handle_request(&self, _ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let result = super::index_models::StatusJsonResult::new(self.app.as_ref()).await;
        return HttpOutput::as_json(result).into_ok_result(true).into();
    }
}
