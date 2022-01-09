use async_trait::async_trait;
use std::sync::Arc;

use my_http_server::{
    middlewares::{
        controllers::{
            actions::{GetAction, PostAction},
            documentation::HttpActionDescription,
        },
        swagger::types::{HttpInputParameter, HttpParameterInputSource, HttpParameterType},
    },
    HttpContext, HttpFailResult, HttpOkResult, WebContentType,
};

use crate::app::AppContext;

use super::models::{JsonTopicResult, JsonTopicsResult};

pub struct TopicsController {
    app: Arc<AppContext>,
}

impl TopicsController {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait]
impl GetAction for TopicsController {
    fn get_controller_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            name: "Topics",
            description: "Get list of topics",
            out_content_type: WebContentType::Json,
        }
        .into()
    }

    fn get_in_parameters_description(&self) -> Option<Vec<HttpInputParameter>> {
        None
    }
    async fn handle_request(&self, _: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let topics = self.app.topic_list.get_all().await;

        let mut response: Vec<JsonTopicResult> = Vec::new();

        for topic in topics {
            let item = JsonTopicResult::new(topic.as_ref()).await;

            response.push(item);
        }

        let root = JsonTopicsResult { items: response };

        HttpOkResult::create_json_response(root).into()
    }
}

#[async_trait]
impl PostAction for TopicsController {
    fn get_controller_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            name: "Topics",
            description: "Create topic",
            out_content_type: WebContentType::Json,
        }
        .into()
    }

    fn get_in_parameters_description(&self) -> Option<Vec<HttpInputParameter>> {
        Some(vec![HttpInputParameter {
            name: "topicId".to_string(),
            param_type: HttpParameterType::String,
            description: "Id of topic".to_string(),
            source: HttpParameterInputSource::Query,
            required: true,
        }])
    }

    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let form_data = ctx.get_form_data().await?;
        let topics_id = form_data.get_required_string_parameter("topicId")?;

        crate::operations::publisher::create_topic_if_not_exists(self.app.clone(), None, topics_id)
            .await;

        let result = HttpOkResult::Text {
            text: "Topic is created".to_string(),
        };

        Ok(result)
    }
}
