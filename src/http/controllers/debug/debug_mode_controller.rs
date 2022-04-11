use super::{super::contracts::response, models::EnableDebugInputModel};
use crate::app::AppContext;
use async_trait::async_trait;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use my_http_server_controllers::controllers::{
    actions::{DeleteAction, PostAction},
    documentation::HttpActionDescription,
};
use std::sync::Arc;
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
    fn get_route(&self) -> &str {
        "/Debug/Enable"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: "Debug",
            description: "Enable debug mode for specific queue",

            input_params: EnableDebugInputModel::get_input_params().into(),
            results: vec![
                response::empty("Debug mode is enabled"),
                response::topic_or_queue_not_found(),
            ],
        }
        .into()
    }

    async fn handle_request(&self, ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let input_data = EnableDebugInputModel::parse_http_input(ctx).await?;

        self.app
            .set_debug_topic_and_queue(input_data.topic_id.as_ref(), input_data.queue_id.as_ref())
            .await;

        HttpOutput::Empty.into_ok_result(true).into()
    }
}

#[async_trait]
impl DeleteAction for DebugModeController {
    fn get_route(&self) -> &str {
        "/Debug/Disable"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: "Debug",
            description: "Disable debug mode",
            input_params: None,
            results: vec![response::empty("Debug mode is disabled")],
        }
        .into()
    }

    async fn handle_request(&self, _ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        self.app.disable_debug_topic_and_queue().await;

        HttpOutput::Empty.into_ok_result(true).into()
    }
}
