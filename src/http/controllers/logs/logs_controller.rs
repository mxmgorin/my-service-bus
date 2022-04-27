use async_trait::async_trait;
use my_http_server_controllers::controllers::{
    actions::GetAction, documentation::HttpActionDescription,
};
use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};
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
    fn get_route(&self) -> &str {
        "/Logs"
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

    async fn handle_request(&self, _ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let mut sw = StopWatch::new();
        sw.start();
        let logs = self.app.logs.get().await;

        return super::renderers::compile_result("logs", logs, sw);
    }
}
