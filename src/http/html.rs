use my_http_utils::{HttpOkResult, WebContentType};

use crate::app::AppContext;

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

pub fn index_page_content(app: &AppContext) -> HttpOkResult {
    let content = format!(
        r###"<html><head><title>{} MyServiceBus</title>
        <link href="/css/bootstrap.css" rel="stylesheet" type="text/css" />
        <link href="/css/site.css?ver={rnd}" rel="stylesheet" type="text/css" />
        <script src="/js/jquery.js"></script><script src="/js/app.js?ver={rnd}"></script>
        </head><body></body></html>"###,
        ver = crate::app::APP_VERSION,
        rnd = app.process_id
    );

    HttpOkResult::Content {
        content_type: Some(WebContentType::Html),
        content: content.into_bytes(),
    }
}
