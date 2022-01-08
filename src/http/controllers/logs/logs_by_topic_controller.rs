use async_trait::async_trait;
use my_http_utils::{HttpContext, HttpFailResult, HttpOkResult, WebContentType};
use rust_extensions::{StopWatch, StringBuilder};
use std::sync::Arc;

use crate::{app::AppContext, http::middlewares::controllers::actions::GetAction};

pub struct LogsByTopicController {
    app: Arc<AppContext>,
}

impl LogsByTopicController {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait]
impl GetAction for LogsByTopicController {
    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let topic_name = ctx.get_value_from_path_optional("TopicId")?;

        if topic_name.is_none() {
            return render_select_topic(self.app.as_ref()).await;
        }

        let topic_name = topic_name.unwrap();

        let mut sw = StopWatch::new();
        sw.start();
        let logs_result = self.app.logs.get_by_topic(topic_name).await;

        match logs_result {
            Some(logs) => super::renderers::compile_result("logs by topic", logs, sw),
            None => {
                sw.pause();

                let content = format!(
                    "Result compiled in: {:?}. No log recods for the topic '{}'",
                    sw.duration(),
                    topic_name
                );

                Ok(HttpOkResult::Content {
                    content_type: Some(WebContentType::Text),
                    content: content.into_bytes(),
                })
            }
        }
    }
}

async fn render_select_topic(app: &AppContext) -> Result<HttpOkResult, HttpFailResult> {
    let mut sb = StringBuilder::new();

    sb.append_line("<h1>Please, select topic to show logs</h1>");

    for topic in app.topic_list.get_all().await {
        let line = format!(
            "<a class='btn btn-sm btn-outline-primary' href='/logs/topic/{topic_id}'>{topic_id}</a>",
            topic_id = topic.topic_id
        );
        sb.append_line(line.as_str())
    }

    Ok(crate::http::html::compile(
        "Select topic to show logs".to_string(),
        sb.to_string_utf8().unwrap(),
    ))
}
