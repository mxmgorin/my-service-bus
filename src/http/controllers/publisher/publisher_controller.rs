use crate::http::controllers::{consts::*, extensions::HttpContextExtensions};
use async_trait::async_trait;
use my_service_bus_tcp_shared::MessageToPublishTcpContract;
use std::sync::Arc;

use my_http_server::{
    middlewares::controllers::{
        actions::{HttpStructsProvider, PostAction},
        documentation::{
            data_types::{ArrayElement, HttpDataProperty, HttpDataType, HttpObjectType},
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

const MESSAGE_HEADER_CONTRACT_ID: &str = "MessageHeaderContract";

const MESSAGE_TO_PUBLISH_CONTRACT_ID: &str = "MessageToPublishContract";

impl HttpStructsProvider for PublisherController {
    fn get(&self) -> Vec<HttpObjectType> {
        vec![
            HttpObjectType {
                struct_id: MESSAGE_HEADER_CONTRACT_ID.to_string(),
                properties: vec![
                    HttpDataProperty::new("key", HttpDataType::as_string(), true),
                    HttpDataProperty::new("value", HttpDataType::as_string(), true),
                ],
            },
            HttpObjectType {
                struct_id: MESSAGE_TO_PUBLISH_CONTRACT_ID.to_string(),
                properties: vec![
                    HttpDataProperty::new(
                        "headers",
                        HttpDataType::ArrayOf(ArrayElement::Object {
                            struct_id: MESSAGE_HEADER_CONTRACT_ID.to_string(),
                        }),
                        true,
                    ),
                    HttpDataProperty::new("base64Message", HttpDataType::as_string(), true),
                ],
            },
        ]
    }
}

#[async_trait]
impl PostAction for PublisherController {
    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            name: "Publish",
            description: "Publish messages to topic",
            input_params: Some(vec![
                get_auth_header_description(),
                get_topic_id_parameter(),
                HttpInputParameter {
                    data_property: HttpDataProperty::new(
                        "messages",
                        HttpDataType::as_array_of_object(MESSAGE_TO_PUBLISH_CONTRACT_ID),
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
