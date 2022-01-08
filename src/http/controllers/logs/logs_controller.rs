use async_trait::async_trait;
use std::sync::Arc;

use my_http_utils::{HttpContext, HttpFailResult, HttpOkResult};
use rust_extensions::StopWatch;

use crate::{app::AppContext, http::middlewares::controllers::actions::GetAction};

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
    async fn handle_request(&self, _ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let mut sw = StopWatch::new();
        sw.start();
        let logs = self.app.logs.get().await;

        return super::renderers::compile_result("logs", logs, sw);
    }
}
