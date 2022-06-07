use async_trait::async_trait;
use my_http_server_controllers::controllers::actions::PostAction;
use my_http_server_controllers::controllers::documentation::out_results::HttpResult;
use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use crate::app::AppContext;

use super::super::contracts::response;
use super::models::{CreateTopicRequestContract, JsonTopicResult, JsonTopicsResult};

use my_http_server_controllers::controllers::{
    actions::GetAction, documentation::HttpActionDescription,
};
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
    fn get_route(&self) -> &str {
        "/Topics"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: "Topics",
            description: "Get list of topics",
            input_params: None,
            results: vec![HttpResult {
                http_code: 200,
                nullable: true,
                description: "List of tables structure".to_string(),
                data_type: JsonTopicsResult::get_http_data_structure().into_http_data_type_array(),
            }],
        }
        .into()
    }

    async fn handle_request(&self, _: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let topics = self.app.topic_list.get_all().await;

        let mut items: Vec<JsonTopicResult> = Vec::new();

        for topic in topics {
            let item = JsonTopicResult::new(topic.as_ref()).await;

            items.push(item);
        }

        let contract = JsonTopicsResult { items };

        HttpOutput::as_json(contract).into_ok_result(true).into()
    }
}

#[async_trait]
impl PostAction for TopicsController {
    fn get_route(&self) -> &str {
        "/Topics/Create"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: "Topics",
            description: "Create topic",

            input_params: CreateTopicRequestContract::get_input_params().into(),

            results: vec![response::empty("Topic is created")],
        }
        .into()
    }

    async fn handle_request(&self, ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let input_data = CreateTopicRequestContract::parse_http_input(ctx).await?;

        crate::operations::publisher::create_topic_if_not_exists(
            &self.app,
            None,
            input_data.topic_id.as_ref(),
        )
        .await?;

        HttpOutput::as_text("Topic is created".to_string())
            .into_ok_result(true)
            .into()
    }
}
