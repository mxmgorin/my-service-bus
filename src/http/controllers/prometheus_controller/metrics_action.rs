use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use my_http_server_controllers::controllers::{
    actions::GetAction, documentation::HttpActionDescription,
};

use crate::app::AppContext;

pub struct MetricsAction {
    app: Arc<AppContext>,
}

impl MetricsAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl GetAction for MetricsAction {
    fn get_route(&self) -> &str {
        "/metrics"
    }
    fn get_description(&self) -> Option<HttpActionDescription> {
        None
    }

    async fn handle_request(&self, _ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let result = self.app.prometheus.build();

        HttpOutput::Content {
            headers: None,
            content_type: None,
            content: result,
        }
        .into_ok_result(true)
        .into()
    }
}
