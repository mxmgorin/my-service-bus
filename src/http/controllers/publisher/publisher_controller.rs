use crate::http::controllers::extensions::HttpContextExtensions;
use async_trait::async_trait;
use my_service_bus_tcp_shared::MessageToPublishTcpContract;
use std::sync::Arc;

use my_http_server::{
    middlewares::controllers::{
        actions::PostAction,
        documentation::{
            data_types::{ArrayElement, HttpDataType, HttpField, HttpObjectStructure},
            in_parameters::{HttpInputParameter, HttpParameterInputSource},
            HttpActionDescription,
        },
    },
    HttpContext, HttpFailResult, HttpOkResult,
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
    fn get_additional_types(&self) -> Option<Vec<HttpObjectStructure>> {
        None
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: "Publish",
            description: "Publish messages to topic",
            input_params: Some(vec![
                super::super::contracts::input_parameters::auth_header(),
                super::super::contracts::input_parameters::topic_id(),
                HttpInputParameter {
                    field: HttpField::new(
                        "messages",
                        HttpDataType::ArrayOf(ArrayElement::Object(HttpObjectStructure {
                            struct_id: "MessageToPublishContract".to_string(),
                            fields: vec![message_headers_contract(), publish_message_contract()],
                        })),
                        true,
                    ),

                    description: "Messages to publish".to_string(),
                    source: HttpParameterInputSource::Body,
                    required: true,
                },
            ]),
            results: vec![],
        }
        .into()
    }

    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let query = ctx.get_query_string()?;
        let topic_id = query.get_required_string_parameter("topicId")?;

        let session_id =
            ctx.get_required_header(super::super::contracts::input_parameters::AUTH_HEADER_NAME)?;
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

fn message_headers_contract() -> HttpField {
    let object_structure = HttpObjectStructure {
        struct_id: "MessageHeadersContract".to_string(),
        fields: vec![
            HttpField::new("key", HttpDataType::as_string(), true),
            HttpField::new("value", HttpDataType::as_string(), true),
        ],
    };

    HttpField {
        name: "headers".to_string(),
        data_type: HttpDataType::ArrayOf(ArrayElement::Object(object_structure)),
        required: false,
    }
}

fn publish_message_contract() -> HttpField {
    HttpField {
        name: "base64Message".to_string(),
        data_type: HttpDataType::as_string(),
        required: true,
    }
}
