use super::super::contracts::response;
use crate::app::AppContext;
use async_trait::async_trait;
use my_http_server::middlewares::controllers::{
    actions::{DeleteAction, PostAction},
    documentation::{
        data_types::{HttpDataType, HttpField, HttpObjectStructure},
        in_parameters::{HttpInputParameter, HttpParameterInputSource},
        HttpActionDescription,
    },
};
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};
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
    fn get_additional_types(&self) -> Option<Vec<HttpObjectStructure>> {
        None
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: "Debug",
            description: "Enable debug mode for specific queue",

            input_params: Some(vec![
                HttpInputParameter {
                    field: HttpField::new("topicId", HttpDataType::as_string(), true),
                    description: "Id of topic".to_string(),
                    source: HttpParameterInputSource::Query,
                    required: true,
                },
                HttpInputParameter {
                    field: HttpField::new("queueId", HttpDataType::as_string(), true),
                    description: "Id of queue".to_string(),
                    source: HttpParameterInputSource::Query,
                    required: true,
                },
            ]),
            results: vec![response::empty("Debug mode is enabled")],
        }
        .into()
    }

    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let query_string = ctx.get_query_string()?;

        let topic_id = query_string.get_required_string_parameter("topicId")?;
        let queue_id = query_string.get_required_string_parameter("queueId")?;

        self.app.set_debug_topic_and_queue(topic_id, queue_id).await;

        Ok(HttpOkResult::Empty)
    }
}

#[async_trait]
impl DeleteAction for DebugModeController {
    fn get_additional_types(&self) -> Option<Vec<HttpObjectStructure>> {
        None
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

    async fn handle_request(&self, _ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        self.app.disable_debug_topic_and_queue().await;

        Ok(HttpOkResult::Text {
            text: "Ok".to_string(),
        })
    }
}
