use async_trait::async_trait;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use my_http_server_controllers::controllers::{
    actions::GetAction, documentation::HttpActionDescription,
};
use std::sync::Arc;

use crate::app::AppContext;

use super::models::GetOnDeliveryInputModel;

pub struct OnDeliveryController {
    app: Arc<AppContext>,
}

impl OnDeliveryController {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait]
impl GetAction for OnDeliveryController {
    fn get_route(&self) -> &str {
        "/Debug/OnDelivery"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: "Debug",
            description: "Show messages on delivery",

            input_params: GetOnDeliveryInputModel::get_input_params().into(),
            results: super::super::contracts::response::text("Ids of subscribers on delivery"),
        }
        .into()
    }

    async fn handle_request(&self, ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let input_model = GetOnDeliveryInputModel::parse_http_input(ctx).await?;

        let topic = self.app.topic_list.get(input_model.topic_id).await;
        if topic.is_none() {
            return Err(HttpFailResult::as_not_found(
                "Topic not found".to_string(),
                false,
            ));
        }

        let topic = topic.unwrap();

        let ids = {
            let topic_data = topic.get_access("debug.get_on_delivery").await;

            let queue = topic_data.queues.get(input_model.queue_id);

            if queue.is_none() {
                return Err(HttpFailResult::as_not_found(
                    "Queue not found".to_string(),
                    false,
                ));
            }

            let queue = queue.unwrap();

            queue.get_messages_on_delivery(input_model.subscriber_id)
        };

        HttpOutput::as_text(format!("{:?}", ids))
            .into_ok_result(true)
            .into()
    }
}
