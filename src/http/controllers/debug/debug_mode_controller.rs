use crate::app::AppContext;
use async_trait::async_trait;
use my_http_server::middlewares::controllers::{
    actions::{DeleteAction, PostAction},
    documentation::{
        HttpActionDescription, HttpInputParameter, HttpParameterInputSource, HttpParameterType,
    },
};
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, WebContentType};
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
    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            name: "Debug",
            description: "Enable debug mode for specific queue",
            out_content_type: WebContentType::Json,
            input_params: Some(vec![
                HttpInputParameter {
                    name: "topicId".to_string(),
                    param_type: HttpParameterType::String,
                    description: "Id of topic".to_string(),
                    source: HttpParameterInputSource::Query,
                    required: true,
                },
                HttpInputParameter {
                    name: "queueId".to_string(),
                    param_type: HttpParameterType::String,
                    description: "Id of queue".to_string(),
                    source: HttpParameterInputSource::Query,
                    required: true,
                },
            ]),
        }
        .into()
    }

    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let query_string = ctx.get_query_string()?;

        let topic_id = query_string.get_required_string_parameter("topicId")?;
        let queue_id = query_string.get_required_string_parameter("queueId")?;

        self.app.set_debug_topic_and_queue(topic_id, queue_id).await;

        Ok(HttpOkResult::Text {
            text: "Ok".to_string(),
        })
    }
}

#[async_trait]
impl DeleteAction for DebugModeController {
    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            name: "Debug",
            description: "Disable debug mode",
            out_content_type: WebContentType::Json,
            input_params: None,
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
