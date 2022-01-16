use async_trait::async_trait;
use my_http_server::{
    middlewares::controllers::{
        actions::PostAction,
        documentation::{
            data_types::{HttpDataType, HttpField, HttpObjectStructure},
            in_parameters::{HttpInputParameter, HttpParameterInputSource},
            out_results::HttpResult,
            HttpActionDescription,
        },
    },
    HttpContext, HttpFailResult, HttpOkResult,
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
    fn get_additional_types(&self) -> Option<Vec<HttpObjectStructure>> {
        None
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: "Greeting",
            description: "Issue new Http Session",

            input_params: Some(vec![
                HttpInputParameter {
                    field: HttpField::new("name", HttpDataType::as_string(), true),

                    description: "Name of client application".to_string(),
                    source: HttpParameterInputSource::Query,
                    required: true,
                },
                HttpInputParameter {
                    field: HttpField::new("version", HttpDataType::as_string(), true),
                    description: "Version of client application".to_string(),
                    source: HttpParameterInputSource::Query,
                    required: true,
                },
            ]),

            results: vec![HttpResult {
                http_code: 200,
                nullable: false,
                description: "Session description".to_string(),
                data_type: HttpDataType::Object(
                    HttpObjectStructure::new("SessionContractResponse")
                        .with_string_field("session", true),
                ),
            }],
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
