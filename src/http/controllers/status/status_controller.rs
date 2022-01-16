use async_trait::async_trait;
use std::sync::Arc;

use my_http_server::{
    middlewares::controllers::{
        actions::GetAction,
        documentation::{data_types::HttpObjectStructure, HttpActionDescription},
    },
    HttpContext, HttpFailResult, HttpOkResult,
};

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
    fn get_additional_types(&self) -> Option<Vec<HttpObjectStructure>> {
        None
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: "Status",
            description: "Get status of application",
            input_params: None,
            results: super::super::contracts::response::object("Objects snapshot"),
        }
        .into()
    }

    async fn handle_request(&self, _ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let result = super::index_models::StatusJsonResult::new(self.app.as_ref()).await;
        return HttpOkResult::create_json_response(result).into();
    }
}
