use crate::{
    app::AppContext,
    http::{http_ctx::HttpContext, HttpFailResult, HttpOkResult},
    operations,
};

pub async fn delete(app: &AppContext, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string();

    let id: i64 = query.get_query_required_parameter("id")?;

    let result = operations::sessions::disconnect(app, id).await;

    if result.is_none() {
        return Err(HttpFailResult::as_not_found(format!(
            "Session {} is not found",
            id
        )));
    }

    let result = HttpOkResult::Text {
        text: "Session is removed".to_string(),
    };

    Ok(result)
}
