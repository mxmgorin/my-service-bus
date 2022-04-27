use async_trait::async_trait;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use my_http_server_controllers::controllers::{
    actions::GetAction, documentation::HttpActionDescription,
};
use rust_extensions::StringBuilder;
use std::sync::Arc;

use crate::app::AppContext;

pub struct LocksController {
    app: Arc<AppContext>,
}

impl LocksController {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait]
impl GetAction for LocksController {
    fn get_route(&self) -> &str {
        "/Locks"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: "Debug",
            description: "Show current locks",
            input_params: None,
            results: super::super::contracts::response::text("List of locks"),
        }
        .into()
    }

    async fn handle_request(&self, _ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let topics = self.app.topic_list.get_all().await;

        let mut result = StringBuilder::new();

        result.append_line("Locks:");

        for topic in topics {
            if let Some(lines) = topic.get_locks().await {
                result.append_line("");
                result.append_line(format!("{}", topic.topic_id).as_str());

                for line in lines {
                    result.append_str(line.as_str());
                    result.append_str(";");
                }
            }
        }

        HttpOutput::as_text(format!("{}", result.to_string_utf8().unwrap()))
            .into_ok_result(true)
            .into()
    }
}
