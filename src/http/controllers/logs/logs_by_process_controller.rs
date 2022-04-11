use async_trait::async_trait;
use my_http_server_controllers::controllers::{
    actions::GetAction, documentation::HttpActionDescription,
};
use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput, WebContentType};
use rust_extensions::{StopWatch, StringBuilder};

use crate::app::{logs::SystemProcess, AppContext};

use super::models::ReadLogsByProcessInputModel;

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
    fn get_route(&self) -> &str {
        "/Logs/Process/{processId}"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: "Logs",
            description: "Show Logs of speciefic process",

            input_params: ReadLogsByProcessInputModel::get_input_params().into(),
            results: vec![],
        }
        .into()
    }

    async fn handle_request(&self, ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let input_params = ReadLogsByProcessInputModel::parse_http_input(ctx).await?;

        if input_params.process_id.is_none() {
            return render_select_process().await;
        }

        let process_id = input_params.process_id.unwrap();

        let process = SystemProcess::parse(process_id.as_str());

        if process.is_none() {
            return HttpOutput::Content {
                content_type: Some(WebContentType::Text),
                content: format!("Invalid process name: {}", process_id).into(),
                headers: None,
            }
            .into_ok_result(false)
            .into();
        }

        let process = process.unwrap();

        let mut sw = StopWatch::new();
        sw.start();
        let logs_result = self.app.logs.get_by_process(process).await;

        match logs_result {
            Some(logs) => super::renderers::compile_result("logs by process", logs, sw),
            None => {
                sw.pause();

                HttpOutput::Content {
                    content_type: Some(WebContentType::Text),
                    content: format!(
                        "Result compiled in: {:?}. No log recods for the process '{}'",
                        sw.duration(),
                        process_id
                    )
                    .into_bytes(),
                    headers: None,
                }
                .into_ok_result(false)
                .into()
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
