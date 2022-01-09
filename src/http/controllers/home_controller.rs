use std::sync::Arc;

use async_trait::async_trait;
use my_http_server::{
    middlewares::controllers::{
        actions::GetAction,
        documentation::{HttpActionDescription, HttpInputParameter},
    },
    HttpContext, HttpFailResult, HttpOkResult, WebContentType,
};

use crate::app::AppContext;
pub struct HomeController {
    app: Arc<AppContext>,
}

impl HomeController {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait]
impl GetAction for HomeController {
    fn get_controller_description(&self) -> Option<HttpActionDescription> {
        None
    }

    fn get_in_parameters_description(&self) -> Option<Vec<HttpInputParameter>> {
        None
    }

    async fn handle_request(&self, _ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let content = format!(
            r###"<html><head><title>{} MyServiceBus</title>
            <link href="/css/bootstrap.css" rel="stylesheet" type="text/css" />
            <link href="/css/site.css?ver={rnd}" rel="stylesheet" type="text/css" />
            <script src="/js/jquery.js"></script><script src="/js/app.js?ver={rnd}"></script>
            </head><body></body></html>"###,
            ver = crate::app::APP_VERSION,
            rnd = self.app.process_id
        );

        HttpOkResult::Content {
            content_type: Some(WebContentType::Html),
            content: content.into_bytes(),
        }
        .into()
    }
}
