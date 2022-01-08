use async_trait::async_trait;
use my_http_utils::{HttpContext, HttpFailResult, HttpOkResult};
use std::sync::Arc;

use crate::{app::AppContext, http::middlewares::controllers::actions::GetAction};

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
    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let query_string = ctx.get_query_string()?;

        let topic_id = query_string.get_required_string_parameter("topic")?;
        let queue_id = query_string.get_required_string_parameter("queue")?;
        let subscriber_id = query_string.get_required_parameter::<i64>("subscriberid")?;

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
