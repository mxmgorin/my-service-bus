use app::AppContext;
use my_service_bus_tcp_shared::{ConnectionAttributes, MySbTcpSerializer};
use my_tcp_sockets::TcpServer;
use tcp::socket_loop::TcpServerEvents;

use std::time::Duration;
use std::{net::SocketAddr, sync::Arc};

mod app;

mod errors;
mod grpc;
mod http;
mod metric_data;
mod operations;
mod persistence;
mod queue_subscribers;
mod queues;
mod sessions;
mod settings;
mod tcp;
mod timers;
mod topics;
mod utils;
pub mod persistence_grpc {
    tonic::include_proto!("persistence");
}

#[tokio::main]
async fn main() {
    let settings = crate::settings::read().await;

    let app = Arc::new(AppContext::new(&settings));

    let mut tasks = Vec::new();

    tasks.push(tokio::task::spawn(crate::operations::initialization::init(
        app.clone(),
    )));

    let tcp_server = TcpServer::new(
        "MySbTcpServer".to_string(),
        SocketAddr::from(([0, 0, 0, 0], 6421)),
    );

    tcp_server
        .start(
            app.clone(),
            Arc::new(|| -> MySbTcpSerializer {
                let attrs = ConnectionAttributes::new(0);
                MySbTcpSerializer::new(attrs)
            }),
            Arc::new(TcpServerEvents::new(app.clone())),
        )
        .await;

    tasks.push(tokio::task::spawn(http::http_server::start(
        SocketAddr::from(([0, 0, 0, 0], 6123)),
        app.clone(),
    )));

    tasks.push(tokio::task::spawn(crate::timers::start(app.clone())));

    signal_hook::flag::register(
        signal_hook::consts::SIGTERM,
        app.states.shutting_down.clone(),
    )
    .unwrap();

    shut_down_task(app).await;

    for task in tasks {
        task.await.unwrap();
    }
}

async fn shut_down_task(app: Arc<AppContext>) {
    let duration = Duration::from_secs(1);

    while !app.states.is_shutting_down() {
        tokio::time::sleep(duration).await;
    }

    println!("Shut down detected. Waiting for 1 second to deliver all messages");
    tokio::time::sleep(duration).await;

    crate::app::shutdown::execute(app).await;
}
