use crate::http::controllers::extensions::HttpContextExtensions;
use my_http_server_controllers::controllers::{
    actions::PostAction, documentation::HttpActionDescription,
};
use my_service_bus_tcp_shared::MessageToPublishTcpContract;
use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use crate::app::AppContext;

use super::contracts::PublishMessageHttpInput;

pub struct PublisherController {
    app: Arc<AppContext>,
}

impl PublisherController {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl PostAction for PublisherController {
    fn get_route(&self) -> &str {
        "/Publish"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: "Publish",
            description: "Publish messages to topic",
            input_params: PublishMessageHttpInput::get_input_params().into(),
            results: vec![],
        }
        .into()
    }

    async fn handle_request(&self, ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let http_input = PublishMessageHttpInput::parse_http_input(ctx).await?;

        let session = self
            .app
            .get_http_session(http_input.authorization.as_str())
            .await?;

        let mut messages_to_publish = Vec::new();

        let mut content_size = 0;

        for mut msg_in_json in http_input.messages {
            let msg = MessageToPublishTcpContract {
                headers: msg_in_json.get_headers(),
                content: msg_in_json.get_content()?,
            };

            content_size += msg.content.len();

            messages_to_publish.push(msg);
        }

        crate::operations::publisher::publish(
            self.app.clone(),
            http_input.topic_id,
            messages_to_publish,
            false,
            session.id,
        )
        .await?;

        let http_session = session.connection.unwrap_as_http();

        http_session.update_written_amount(content_size);

        HttpOutput::Empty.into_ok_result(true).into()
    }
}
