use async_trait::async_trait;
use std::sync::Arc;

use my_http_utils::{HttpContext, HttpFailResult, HttpOkResult, WebContentType};
use rust_extensions::{StopWatch, StringBuilder};

use crate::{
    app::{logs::SystemProcess, AppContext},
    http::middlewares::{
        controllers::{actions::GetAction, documentation::HttpActionDescription},
        swagger::types::{
            SwaggerInputParameter, SwaggerParameterInputSource, SwaggerParameterType,
        },
    },
};

pub struct LogsByProcessController {
    app: Arc<AppContext>,
}

impl LogsByProcessController {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait]
impl GetAction for LogsByProcessController {
    fn get_controller_description(&self) -> HttpActionDescription {
        HttpActionDescription {
            name: "Logs",
            description: "Show Logs of speciefic process",
            out_content_type: WebContentType::Json,
        }
    }

    fn get_in_parameters_description(&self) -> Option<Vec<SwaggerInputParameter>> {
        Some(vec![SwaggerInputParameter {
            name: "processId".to_string(),
            param_type: SwaggerParameterType::String,
            description: "Id of process".to_string(),
            source: SwaggerParameterInputSource::Path,
            required: false,
        }])
    }

    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let process_name = ctx.get_value_from_path_optional("processId")?;

        if process_name.is_none() {
            return render_select_process().await;
        }

        let process_name = process_name.unwrap();

        let process = SystemProcess::parse(process_name);

        if process.is_none() {
            return Ok(HttpOkResult::Content {
                content_type: Some(WebContentType::Text),
                content: format!("Invalid process name: {}", process_name).into(),
            });
        }

        let process = process.unwrap();

        let mut sw = StopWatch::new();
        sw.start();
        let logs_result = self.app.logs.get_by_process(process).await;

        match logs_result {
            Some(logs) => super::renderers::compile_result("logs by process", logs, sw),
            None => {
                sw.pause();

                Ok(HttpOkResult::Content {
                    content_type: Some(WebContentType::Text),
                    content: format!(
                        "Result compiled in: {:?}. No log recods for the process '{}'",
                        sw.duration(),
                        process_name
                    )
                    .into_bytes(),
                })
            }
        }
    }
}

async fn render_select_process() -> Result<HttpOkResult, HttpFailResult> {
    let mut sb = StringBuilder::new();

    sb.append_line("<h1>Please, select process to show logs</h1>");

    for process in &SystemProcess::iterate() {
        let line = format!(
            "<a class='btn btn-sm btn-outline-primary' href='/logs/process/{process:?}'>{process:?}</a>",
            process = process
        );
        sb.append_line(line.as_str())
    }

    Ok(crate::http::html::compile(
        "Select topic to show logs".to_string(),
        sb.to_string_utf8().unwrap(),
    ))
}
