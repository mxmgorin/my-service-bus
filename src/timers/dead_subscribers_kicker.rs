use std::{sync::Arc, time::Duration};

use crate::app::AppContext;

pub async fn start(app: Arc<AppContext>, delivery_timeout_duration: Duration) {
    let duration = Duration::from_secs(10);

    while !app.states.is_initialized() {
        tokio::time::sleep(duration).await;
    }

    println!("Kick dead connections timer is started");

    while !app.states.app_is_shutted_down() {
        let handler = tokio::spawn(kick_them(app.clone(), delivery_timeout_duration)).await;
        if let Err(err) = handler {
            app.logs
                .add_fatal_error(
                    crate::app::logs::SystemProcess::Timer,
                    "dead_subscribers_kicker_loop".to_string(),
                    format!("{:?}", err),
                )
                .await;
        }

        tokio::time::sleep(duration).await;
    }
}

pub async fn kick_them(app: Arc<AppContext>, delivery_timeout_duration: Duration) {
    let join_handle = tokio::spawn(execute(app.clone(), delivery_timeout_duration)).await;

    if let Err(err) = join_handle {
        app.logs
            .add_fatal_error(
                crate::app::logs::SystemProcess::Timer,
                "dead_subscribers_kicker".to_string(),
                format!("{:?}", err),
            )
            .await;
    }
}

async fn execute(app: Arc<AppContext>, delivery_timeout_duration: Duration) {
    let topics = app.topic_list.get_all().await;

    let process_id = app.process_id_generator.get_process_id().await;

    for topic in topics {
        let queues = topic.get_all_queues().await;

        for queue in queues {
            let dead_subscribers = queue
                .find_subscribers_dead_on_delivery(delivery_timeout_duration)
                .await;

            if let Some(dead_subscribers) = dead_subscribers {
                for dead_subscriber in dead_subscribers {
                    app.logs
                        .add_error(
                            Some(topic.topic_id.to_string()),
                            crate::app::logs::SystemProcess::Timer,
                            "Dead subscribers detector".to_string(),
                            format!(
                                "Kicking Connection {} with dead subscriber {}",
                                dead_subscriber.session.get_name(process_id).await,
                                dead_subscriber.subscriber_id
                            ),
                            Some(format!("{:?}", dead_subscriber.duration)),
                        )
                        .await;

                    crate::operations::sessions::disconnect(
                        process_id,
                        app.as_ref(),
                        dead_subscriber.session,
                    )
                    .await;
                }
            }
        }
    }
}
