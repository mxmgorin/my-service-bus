use async_trait::async_trait;
use my_http_server::{
    middlewares::controllers::{
        actions::PostAction,
        documentation::{
            HttpActionDescription, HttpInputParameter, HttpParameterInputSource, HttpParameterType,
        },
    },
    HttpContext, HttpFailResult, HttpOkResult, WebContentType,
};
use std::sync::Arc;

use crate::{app::AppContext, sessions::HttpConnectionData};

use super::models::GreetingJsonResult;

pub struct GreetingController {
    app: Arc<AppContext>,
}

impl GreetingController {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait]
impl PostAction for GreetingController {
    fn get_controller_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            name: "Greeting",
            description: "Issue new Http Session",
            out_content_type: WebContentType::Json,
            input_params: Some(vec![
                HttpInputParameter {
                    name: "name".to_string(),
                    param_type: HttpParameterType::String,
                    description: "Name of client application".to_string(),
                    source: HttpParameterInputSource::Query,
                    required: true,
                },
                HttpInputParameter {
                    name: "version".to_string(),
                    param_type: HttpParameterType::String,
                    description: "Version of client application".to_string(),
                    source: HttpParameterInputSource::Query,
                    required: true,
                },
            ]),
        }
        .into()
    }

    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let query = ctx.get_query_string()?;

        let app_name = query.get_required_string_parameter("name")?;
        let app_version = query.get_required_string_parameter("version")?;

        let id = uuid::Uuid::new_v4().to_string();

        let data = HttpConnectionData::new(
            id.to_string(),
            app_name.to_string(),
            app_version.to_string(),
            ctx.get_ip().get_real_ip().to_string(),
        );

        self.app.sessions.add_http(data).await;

        let result = GreetingJsonResult { session: id };

        HttpOkResult::create_json_response(result).into()
    }
}
