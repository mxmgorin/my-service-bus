use async_trait::async_trait;
use std::sync::Arc;

use my_http_utils::{HttpContext, HttpFailResult, HttpOkResult, WebContentType};

use crate::{
    app::AppContext,
    http::middlewares::{
        controllers::{actions::GetAction, documentation::HttpActionDescription},
        swagger::types::SwaggerInputParameter,
    },
};

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
    fn get_controller_description(&self) -> HttpActionDescription {
        HttpActionDescription {
            name: "Status",
            description: "Get status of application",
            out_content_type: WebContentType::Json,
        }
    }

    fn get_in_parameters_description(&self) -> Option<Vec<SwaggerInputParameter>> {
        None
    }

    async fn handle_request(&self, _ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let result = super::index_models::StatusJsonResult::new(self.app.as_ref()).await;
        return HttpOkResult::create_json_response(result).into();
    }
}
