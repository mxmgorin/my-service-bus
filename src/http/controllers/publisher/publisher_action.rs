use crate::http::controllers::extensions::HttpContextExtensions;

use my_http_server_swagger::http_route;
use my_service_bus_tcp_shared::MessageToPublishTcpContract;
use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use crate::app::AppContext;

use super::contracts::PublishMessageHttpInput;

#[http_route(
    method: "POST",
    route: "/Publish",
    controller: "Publish",
    description: "Publish messages to topic",
    input_data: "PublishMessageHttpInput",
    ok_result_description: "Messages are published"
)]
pub struct PublisherController {
    app: Arc<AppContext>,
}

impl PublisherController {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &PublisherController,
    http_input: PublishMessageHttpInput,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let session = action
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
        action.app.clone(),
        http_input.topic_id.as_str(),
        messages_to_publish,
        false,
        session.id,
    )
    .await?;

    let http_session = session.connection.unwrap_as_http();

    http_session.update_written_amount(content_size);

    HttpOutput::Empty.into_ok_result(true).into()
}
