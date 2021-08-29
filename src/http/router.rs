use hyper::{Body, Method, Request};

use crate::app::AppContext;
use std::sync::Arc;

use super::{
    http_ctx::HttpContext, http_fail::HttpFailResult, http_ok::HttpOkResult,
    web_content_type::WebContentType,
};

pub async fn route_requests(
    req: Request<Body>,
    app: Arc<AppContext>,
) -> Result<HttpOkResult, HttpFailResult> {
    let ctx = HttpContext::new(req);

    let path = ctx.get_path();

    match (ctx.get_method(), path.as_str()) {
        (&Method::GET, "/logs") => {
            return super::controllers::logs::get(app.as_ref()).await;
        }

        (&Method::GET, "/status") => {
            return super::controllers::status::index::get(app.as_ref()).await;
        }

        (&Method::GET, "/metrics") => {
            return super::controllers::metrics::get(app.as_ref()).await;
        }

        (&Method::DELETE, "/connections/kicktcpconnection") => {
            return super::controllers::connections::delete(app.as_ref(), ctx).await;
        }

        (&Method::DELETE, "/queues") => {
            return super::controllers::queues::delete(app.as_ref(), ctx).await;
        }

        (&Method::POST, "/queues/setmessageid") => {
            return super::controllers::queues::set_message_id(app.as_ref(), ctx).await;
        }

        (&Method::GET, "/topics") => {
            return super::controllers::topics::get(app.as_ref()).await;
        }

        _ => {}
    };

    if path.starts_with("/logs/topic") {
        return super::controllers::logs::get_by_topic(app.as_ref(), &path).await;
    }

    if path.starts_with("/logs/process") {
        return super::controllers::logs::get_by_process(app.as_ref(), &path).await;
    }

    if path.starts_with("/swagger") {
        return super::swagger::handle_request(ctx).await;
    }

    if path.starts_with("/css") {
        return super::files::get_content_from_file(path.as_str(), Some(WebContentType::Css)).await;
    }

    if path.starts_with("/img") {
        return super::files::get_content_from_file(
            path.as_str(),
            Some(WebContentType::detect_by_extension(path.as_str())),
        )
        .await;
    }

    if path.starts_with("/js") {
        return super::files::get_content_from_file(
            path.as_str(),
            Some(WebContentType::JavaScript),
        )
        .await;
    }

    if path == "/" {
        return Ok(get_index_page_content(app.as_ref()));
    }

    return Err(HttpFailResult::as_not_found("Not Found".to_string()));
}

fn get_index_page_content(app: &AppContext) -> HttpOkResult {
    let content = format!(
        r###"<html><head><title>{} MyNoSQLServer</title>
        <link href="/css/bootstrap.css" rel="stylesheet" type="text/css" />
        <link href="/css/site.css?ver={rnd}" rel="stylesheet" type="text/css" />
        <script src="/js/jquery.js"></script><script src="/js/app.js?ver={rnd}"></script>
        </head><body></body></html>"###,
        ver = crate::app::APP_VERSION,
        rnd = app.process_id
    );

    HttpOkResult::Content {
        content_type: Some(crate::http::web_content_type::WebContentType::Html),
        content: content.into_bytes(),
    }
}
