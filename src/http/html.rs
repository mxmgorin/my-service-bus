use my_http_server::{HttpOkResult, WebContentType};

pub fn compile(title: String, body: String) -> HttpOkResult {
    let content = format!(
        r###"<html><head><title>{ver} MyServiceBus {title}</title>
        <link href="/css/bootstrap.css" rel="stylesheet" type="text/css" />
        </head><body>{body}</body></html>"###,
        ver = crate::app::APP_VERSION,
        title = title,
        body = body
    );

    HttpOkResult::Content {
        content_type: Some(WebContentType::Html),
        content: content.into_bytes(),
    }
}
