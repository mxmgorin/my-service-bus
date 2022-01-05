use crate::{
    app::AppContext,
    http::{http_ctx::HttpContext, HttpFailResult, HttpOkResult},
    sessions::SessionId,
};

pub async fn delete(app: &AppContext, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string();

    let id: SessionId = query.get_query_required_parameter("id")?;

    match app.sessions.get(id).await {
        Some(session) => {
            session.disconnect().await;

            let result = HttpOkResult::Text {
                text: "Session is removed".to_string(),
            };
            Ok(result)
        }
        None => Err(HttpFailResult::as_not_found(format!(
            "Session {} is not found",
            id
        ))),
    }
}
