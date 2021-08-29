use std::collections::HashMap;

use super::{http_ctx::HttpContext, http_fail::HttpFailResult, http_ok::HttpOkResult};

pub async fn handle_request(ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
    let path = ctx.get_path();
    let scheme = ctx.get_scheme();

    let host = ctx.get_host();

    if path == "/swagger" {
        let new_url = format!("{}://{}/swagger/Index.html", scheme, host);
        return Ok(HttpOkResult::Redirect { url: new_url });
    }

    if path == "/swagger/v1/swagger.json" {
        let mut placehloders = HashMap::new();

        placehloders.insert("SCHEME", ctx.get_scheme());

        placehloders.insert("HOST", host.to_string());

        return super::files::serve_file_with_placeholders(path.as_str(), None, &placehloders)
            .await;
    }

    let result = super::files::get_content_from_file(path.as_str(), None).await;

    match result {
        Ok(result) => return Ok(result),
        _ => {
            let new_url = format!("{}://{}/swagger/Index.html", scheme, host);
            return Ok(HttpOkResult::Redirect { url: new_url });
        }
    }
}
