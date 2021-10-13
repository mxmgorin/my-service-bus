use crate::{
    app::AppContext,
    http::{HttpFailResult, HttpOkResult},
};

pub async fn get(app: &AppContext) -> Result<HttpOkResult, HttpFailResult> {
    let result = super::index_models::StatusJsonResult::new(app).await;
    return HttpOkResult::create_json_response(result);
}
