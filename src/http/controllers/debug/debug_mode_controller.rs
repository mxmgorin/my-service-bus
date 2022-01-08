use async_trait::async_trait;
use my_http_utils::{HttpContext, HttpFailResult, HttpOkResult};
use std::sync::Arc;

use crate::{
    app::AppContext,
    http::middlewares::controllers::actions::{DeleteAction, PostAction},
};

pub struct DebugModeController {
    app: Arc<AppContext>,
}

impl DebugModeController {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait]
impl PostAction for DebugModeController {
    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let query_string = ctx.get_query_string()?;

        let topic_id = query_string.get_required_string_parameter("topic")?;
        let queue_id = query_string.get_required_string_parameter("queue")?;

        self.app.set_debug_topic_and_queue(topic_id, queue_id).await;

        Ok(HttpOkResult::Text {
            text: "Ok".to_string(),
        })
    }
}

#[async_trait]
impl DeleteAction for DebugModeController {
    async fn handle_request(&self, _ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        self.app.disable_debug_topic_and_queue().await;

        Ok(HttpOkResult::Text {
            text: "Ok".to_string(),
        })
    }
}
