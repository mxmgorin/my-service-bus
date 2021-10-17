use std::{sync::Arc, time::Duration};

use crate::app::AppContext;

pub async fn start(app: Arc<AppContext>) {
    let duration = Duration::from_secs(10);

    while !app.states.is_initialized() {
        tokio::time::sleep(duration).await;
    }

    app.logs
        .add_info(
            None,
            crate::app::logs::SystemProcess::Timer,
            "Timer by topic".to_string(),
            "Started".to_string(),
        )
        .await;

    while !app.states.app_is_shutted_down() {
        timer_tick(app.clone()).await;
        tokio::time::sleep(duration).await;
    }
}

pub async fn timer_tick(app: Arc<AppContext>) {
    for topic in app.topic_list.get_all().await {
        let message_pages_result =
            tokio::spawn(super::message_pages::execute(app.clone(), topic.clone())).await;

        if let Err(err) = message_pages_result {
            app.logs
                .add_fatal_error(
                    crate::app::logs::SystemProcess::Timer,
                    "message_pages::execute".to_string(),
                    format!("{}", err),
                )
                .await
        }

        let no_subscribers_queue_result =
            tokio::spawn(super::no_subscribers_queues::execute(app.clone(), topic)).await;

        if let Err(err) = no_subscribers_queue_result {
            app.logs
                .add_fatal_error(
                    crate::app::logs::SystemProcess::Timer,
                    "no_subscribers_queues::execute".to_string(),
                    format!("{}", err),
                )
                .await
        }
    }
}
