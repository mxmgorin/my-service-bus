use async_trait::async_trait;
use std::sync::Arc;

use my_http_server::{
    middlewares::controllers::{
        actions::{GetAction, PostAction},
        documentation::{data_types::HttpObjectType, HttpActionDescription},
    },
    HttpContext, HttpFailResult, HttpOkResult,
};

use crate::{app::AppContext, http::controllers::consts::get_topic_id_parameter};

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
    fn get_additional_types(&self) -> Option<Vec<HttpObjectType>> {
        None
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            name: "Topics",
            description: "Get list of topics",
            input_params: None,
            results: vec![],
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
    fn get_additional_types(&self) -> Option<Vec<HttpObjectType>> {
        None
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            name: "Topics",
            description: "Create topic",

            input_params: Some(vec![get_topic_id_parameter()]),

            results: vec![],
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
