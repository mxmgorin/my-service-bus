use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use my_http_server_controllers::controllers::{
    actions::PostAction,
    documentation::{out_results::HttpResult, HttpActionDescription},
};

use crate::{app::AppContext, sessions::HttpConnectionData};

use super::models::{GreetingInputModel, GreetingJsonResult};

pub struct GreetingController {
    app: Arc<AppContext>,
}

impl GreetingController {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl PostAction for GreetingController {
    fn get_route(&self) -> &str {
        "/Greeting"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: "Greeting",
            description: "Issue new Http Session",

            input_params: GreetingInputModel::get_input_params().into(),

            results: vec![HttpResult {
                http_code: 200,
                nullable: false,
                description: "Session description".to_string(),
                data_type: GreetingJsonResult::get_http_data_structure()
                    .into_http_data_type_object(),
            }],
        }
        .into()
    }

    async fn handle_request(&self, ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let ip = ctx.request.get_ip().get_real_ip().to_string();
        let input_data = GreetingInputModel::parse_http_input(ctx).await?;

        let id = uuid::Uuid::new_v4().to_string();

        let data = HttpConnectionData::new(id.to_string(), input_data.name, input_data.version, ip);

        self.app.sessions.add_http(data).await;

        let result = GreetingJsonResult { session: id };

        HttpOutput::as_json(result).into_ok_result(true).into()
    }
}
