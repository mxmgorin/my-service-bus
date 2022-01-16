use async_trait::async_trait;
use std::sync::Arc;

use my_http_server::{
    middlewares::controllers::{
        actions::{GetAction, PostAction},
        documentation::{
            data_types::{ArrayElement, HttpDataType, HttpField, HttpObjectStructure},
            out_results::HttpResult,
            HttpActionDescription,
        },
    },
    HttpContext, HttpFailResult, HttpOkResult,
};

use crate::app::AppContext;

use super::super::contracts::response;
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
    fn get_additional_types(&self) -> Option<Vec<HttpObjectStructure>> {
        None
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: "Topics",
            description: "Get list of topics",
            input_params: None,
            results: vec![list_of_topics_result()],
        }
        .into()
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
    fn get_additional_types(&self) -> Option<Vec<HttpObjectStructure>> {
        None
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: "Topics",
            description: "Create topic",

            input_params: Some(vec![super::super::contracts::input_parameters::topic_id()]),

            results: vec![response::empty("Topic is created")],
        }
        .into()
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

fn list_of_topics_result() -> HttpResult {
    HttpResult {
        http_code: 200,
        nullable: false,
        description: "List of topics".to_string(),
        data_type: HttpDataType::ArrayOf(ArrayElement::Object(HttpObjectStructure {
            struct_id: "TopicDescriptionContract".to_string(),
            fields: topic_description_fields(),
        })),
    }
}

fn topic_description_fields() -> Vec<HttpField> {
    vec![
        HttpField::new("id", HttpDataType::as_string(), true),
        HttpField::new("messageId", HttpDataType::as_long(), true),
    ]
}
