use crate::{
    app::AppContext,
    http::{http_ctx::HttpContext, HttpFailResult, HttpOkResult},
    operations,
};

pub async fn delete(app: &AppContext, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string();

    let id: i64 = query.get_query_required_parameter("id")?;

    let session_result = operations::sessions::disconnect(app, id).await;

    match session_result {
        Some(_) => {
            return Ok(HttpOkResult::Ok);
        }
        None => {
            return Err(HttpFailResult::as_not_found(format!(
                "Session {} is not found",
                &id
            )));
        }
    }
}
