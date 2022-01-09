use async_trait::async_trait;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, WebContentType};
use std::sync::Arc;

use crate::app::AppContext;

use my_http_server::middlewares::controllers::{
    actions::GetAction,
    documentation::{
        HttpActionDescription, HttpInputParameter, HttpParameterInputSource, HttpParameterType,
    },
};

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
    fn get_controller_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            name: "Debug",
            description: "Show messages on delivery",
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
                HttpInputParameter {
                    name: "subscriberId".to_string(),
                    param_type: HttpParameterType::Long,
                    description: "Id of subscriber".to_string(),
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
        let subscriber_id = query_string.get_required_parameter::<i64>("subscriberId")?;

        let topic = self.app.topic_list.get(topic_id).await;
        if topic.is_none() {
            return Err(HttpFailResult::as_not_found(
                "Topic not found".to_string(),
                false,
            ));
        }

        let topic = topic.unwrap();

        let ids = {
            let topic_data = topic.get_access("debug.get_on_delivery").await;

            let queue = topic_data.queues.get(queue_id);

            if queue.is_none() {
                return Err(HttpFailResult::as_not_found(
                    "Queue not found".to_string(),
                    false,
                ));
            }

            let queue = queue.unwrap();

            queue.get_messages_on_delivery(subscriber_id)
        };

        return Ok(HttpOkResult::Text {
            text: format!("{:?}", ids),
        });
    }
}
