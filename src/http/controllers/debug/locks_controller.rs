use async_trait::async_trait;
use my_http_server::{
    middlewares::controllers::documentation::data_types::HttpObjectType, HttpContext,
    HttpFailResult, HttpOkResult,
};
use rust_extensions::StringBuilder;
use std::sync::Arc;

use crate::app::AppContext;

use my_http_server::middlewares::controllers::{
    actions::GetAction, documentation::HttpActionDescription,
};

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
    fn get_additional_types(&self) -> Option<Vec<HttpObjectType>> {
        None
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            name: "Debug",
            description: "Show current locks",
            input_params: None,
            results: super::super::consts::get_text_result(),
        }
        .into()
    }

    async fn handle_request(&self, _ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
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

        return Ok(HttpOkResult::Text {
            text: format!("{}", result.to_string_utf8().unwrap()),
        });
    }
}
