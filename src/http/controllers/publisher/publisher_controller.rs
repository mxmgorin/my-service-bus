use crate::http::controllers::{consts::*, extensions::HttpContextExtensions};
use async_trait::async_trait;
use my_service_bus_tcp_shared::MessageToPublishTcpContract;
use std::sync::Arc;

use my_http_server::{
    middlewares::controllers::{
        actions::PostAction,
        documentation::{
            HttpActionDescription, HttpInputParameter, HttpParameterInputSource, HttpParameterType,
        },
    },
    HttpContext, HttpFailResult, HttpOkResult, WebContentType,
};

use crate::app::AppContext;

use super::contracts::MessageToPublishJsonModel;

pub struct PublisherController {
    app: Arc<AppContext>,
}

impl PublisherController {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait]
impl PostAction for PublisherController {
    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            name: "Publish",
            description: "Publish messages to topic",
            out_content_type: WebContentType::Text,
            input_params: Some(vec![
                HttpInputParameter {
                    name: "topicId".to_string(),
                    param_type: HttpParameterType::String,
                    description: "Id of topic".to_string(),
                    source: HttpParameterInputSource::Query,
                    required: true,
                },
                get_auth_header_description(),
            ]),
        }
        .into()
    }

    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let query = ctx.get_query_string()?;
        let topic_id = query.get_required_string_parameter("topicId")?;

        let session_id = ctx.get_required_header(AUTH_HEADER_NAME)?;
        let session = self.app.get_http_session(session_id).await?;

        let as_json: Vec<MessageToPublishJsonModel> = ctx.get_body_as_json().await?;

        let mut messages_to_publish = Vec::new();

        let mut content_size = 0;

        for mut msg_in_json in as_json {
            let msg = MessageToPublishTcpContract {
                headers: msg_in_json.get_headers(),
                content: msg_in_json.get_content()?,
            };

            content_size += msg.content.len();

            messages_to_publish.push(msg);
        }

        crate::operations::publisher::publish(
            self.app.clone(),
            topic_id,
            messages_to_publish,
            false,
            session.id,
        )
        .await?;

        let http_session = session.connection.unwrap_as_http();

        http_session.update_written_amount(content_size);

        Ok(HttpOkResult::Ok)
    }
}
