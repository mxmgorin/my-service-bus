use async_trait::async_trait;
use std::sync::Arc;

use my_http_server::{
    middlewares::{
        controllers::{actions::GetAction, documentation::HttpActionDescription},
        swagger::types::HttpInputParameter,
    },
    HttpContext, HttpFailResult, HttpOkResult, WebContentType,
};
use rust_extensions::StopWatch;

use crate::app::AppContext;

pub struct LogsController {
    app: Arc<AppContext>,
}

impl LogsController {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait]
impl GetAction for LogsController {
    fn get_controller_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            name: "Logs",
            description: "Show Logs",
            out_content_type: WebContentType::Json,
        }
        .into()
    }
    fn get_in_parameters_description(&self) -> Option<Vec<HttpInputParameter>> {
        None
    }
    async fn handle_request(&self, _ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let mut sw = StopWatch::new();
        sw.start();
        let logs = self.app.logs.get().await;

        return super::renderers::compile_result("logs", logs, sw);
    }
}
