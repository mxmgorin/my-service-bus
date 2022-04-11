use async_trait::async_trait;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use my_http_server_controllers::controllers::{
    actions::{DeleteAction, GetAction, PostAction},
    documentation::{
        data_types::{ArrayElement, HttpDataType, HttpSimpleType},
        out_results::HttpResult,
        HttpActionDescription,
    },
};

use std::sync::Arc;

use super::{super::contracts::response, *};

use crate::app::AppContext;
pub struct QueuesController {
    app: Arc<AppContext>,
}

impl QueuesController {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait]
impl GetAction for QueuesController {
    fn get_route(&self) -> &str {
        "/Queues"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: "Queues",
            description: "Get list of queues",

            input_params: GetListOfQueuesInputContract::get_input_params().into(),
            results: vec![
                HttpResult {
                    http_code: 200,
                    description: "List of queues".to_string(),
                    nullable: false,
                    data_type: HttpDataType::ArrayOf(ArrayElement::SimpleType(
                        HttpSimpleType::String,
                    )),
                },
                response::topic_not_found(),
            ],
        }
        .into()
    }

    async fn handle_request(&self, ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let input_data = GetListOfQueuesInputContract::parse_http_input(ctx).await?;

        let topic = self.app.topic_list.get(input_data.topic_id).await;

        if topic.is_none() {
            return Err(HttpFailResult::as_not_found(
                format!("Topic {} not found", input_data.topic_id),
                false,
            ));
        }

        let topic = topic.unwrap();

        let mut result = Vec::new();

        {
            let topic_data = topic.get_access("http.get_queues").await;
            for queue in topic_data.queues.get_queues() {
                result.push(queue.queue_id.clone());
            }
        }

        HttpOutput::as_json(result).into_ok_result(true).into()
    }
}

#[async_trait]
impl DeleteAction for QueuesController {
    fn get_route(&self) -> &str {
        "/Queues"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: "Queues",
            description: "Delete queue",
            input_params: DeleteQueueInputContract::get_input_params().into(),
            results: vec![
                response::empty("Topic is Deleted"),
                response::topic_or_queue_not_found(),
            ],
        }
        .into()
    }

    async fn handle_request(&self, ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let http_input = DeleteQueueInputContract::parse_http_input(ctx).await?;

        crate::operations::queues::delete_queue(
            self.app.as_ref(),
            http_input.topic_id,
            http_input.queue_id,
        )
        .await?;

        HttpOutput::Empty.into_ok_result(true).into()
    }
}

#[async_trait]
impl PostAction for QueuesController {
    fn get_route(&self) -> &str {
        "/Queues/SetMessageId"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: "Queues",
            description: "Set message id of the queue",

            input_params: SetQueueMessageIdInputContract::get_input_params().into(),
            results: vec![],
        }
        .into()
    }

    async fn handle_request(&self, ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let http_input = SetQueueMessageIdInputContract::parse_http_input(ctx).await?;

        crate::operations::queues::set_message_id(
            self.app.as_ref(),
            http_input.topic_id,
            http_input.queue_id,
            http_input.message_id,
        )
        .await?;

        HttpOutput::Empty.into_ok_result(true).into()
    }
}
