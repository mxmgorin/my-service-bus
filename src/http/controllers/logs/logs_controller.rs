use async_trait::async_trait;
use std::sync::Arc;

use my_http_server::{
    middlewares::controllers::{
        actions::GetAction,
        documentation::{data_types::HttpObjectStructure, HttpActionDescription},
    },
    HttpContext, HttpFailResult, HttpOkResult,
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
    fn get_additional_types(&self) -> Option<Vec<HttpObjectStructure>> {
        None
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: "Logs",
            description: "Show Logs",
            input_params: None,
            results: vec![],
        }
        .into()
    }

    async fn handle_request(&self, _ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let mut sw = StopWatch::new();
        sw.start();
        let logs = self.app.logs.get().await;

        return super::renderers::compile_result("logs", logs, sw);
    }
}
