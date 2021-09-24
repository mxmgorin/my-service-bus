use crate::{
    app::AppContext,
    http::{HttpFailResult, HttpOkResult},
};

pub async fn get(app: &AppContext) -> Result<HttpOkResult, HttpFailResult> {
    let process_id = app.process_id_generator.get_process_id().await;
    let result = super::index_models::StatusJsonResult::new(app, process_id).await;
    return HttpOkResult::create_json_response(result);
}
