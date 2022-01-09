use async_trait::async_trait;
use my_http_utils::{HttpContext, HttpFailResult, HttpOkResult, WebContentType};
use std::sync::Arc;

use crate::{
    app::AppContext,
    http::middlewares::{
        controllers::{
            actions::{DeleteAction, PostAction},
            documentation::HttpActionDescription,
        },
        swagger::types::{
            SwaggerInputParameter, SwaggerParameterInputSource, SwaggerParameterType,
        },
    },
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
    fn get_controller_description(&self) -> HttpActionDescription {
        HttpActionDescription {
            name: "Debug",
            description: "Enable debug mode for specific queue",
            out_content_type: WebContentType::Json,
        }
    }

    fn get_in_parameters_description(&self) -> Option<Vec<SwaggerInputParameter>> {
        Some(vec![
            SwaggerInputParameter {
                name: "topicId".to_string(),
                param_type: SwaggerParameterType::String,
                description: "Id of topic".to_string(),
                source: SwaggerParameterInputSource::Query,
                required: true,
            },
            SwaggerInputParameter {
                name: "queueId".to_string(),
                param_type: SwaggerParameterType::String,
                description: "Id of queue".to_string(),
                source: SwaggerParameterInputSource::Query,
                required: true,
            },
        ])
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
    fn get_controller_description(&self) -> HttpActionDescription {
        HttpActionDescription {
            name: "Debug",
            description: "Disable debug mode",
            out_content_type: WebContentType::Json,
        }
    }

    fn get_in_parameters_description(&self) -> Option<Vec<SwaggerInputParameter>> {
        None
    }

    async fn handle_request(&self, _ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        self.app.disable_debug_topic_and_queue().await;

        Ok(HttpOkResult::Text {
            text: "Ok".to_string(),
        })
    }
}
