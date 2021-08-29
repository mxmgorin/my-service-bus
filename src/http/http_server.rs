use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};

use std::{net::SocketAddr, time::Duration};

use crate::app::AppContext;
use std::sync::Arc;

pub async fn start(addr: SocketAddr, app: Arc<AppContext>) {
    app.logs
        .add_info(
            None,
            crate::app::logs::SystemProcess::System,
            "Starting http server".to_string(),
            addr.to_string(),
        )
        .await;

    let app_to_move = app.clone();

    let make_service = make_service_fn(move |_| {
        let http_app = app_to_move.clone();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |_req| {
                handle_requests(_req, http_app.clone())
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_service);

    let server = server.with_graceful_shutdown(shutdown_signal(app.clone()));

    if let Err(e) = server.await {
        eprintln!("Http Server error: {}", e);
    }
}

pub async fn handle_requests(
    req: Request<Body>,
    app: Arc<AppContext>,
) -> hyper::Result<Response<Body>> {
    let response = super::router::route_requests(req, app).await;

    let response = match response {
        Ok(ok_result) => ok_result.into(),
        Err(fail_result) => fail_result.into(),
    };

    return Ok(response);
}

async fn shutdown_signal(app: Arc<AppContext>) {
    let duration = Duration::from_secs(1);
    while !app.states.is_shutting_down() {
        tokio::time::sleep(duration).await;
    }
}
