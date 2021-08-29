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
        let handle = tokio::task::spawn(timer_tick(app.clone()));

        if let Err(err) = handle.await {
            app.logs
                .add_error(
                    None,
                    crate::app::logs::SystemProcess::Timer,
                    "By topic timer tick".to_string(),
                    "Error during doing By topic one second timer iteration".to_string(),
                    Some(format!("{:?}", err)),
                )
                .await;
        }
        tokio::time::sleep(duration).await;
    }
}

async fn timer_tick(app: Arc<AppContext>) {
    for topic in app.topic_list.get_all().await {
        super::message_pages_loader_and_gc::execute(app.as_ref(), topic.as_ref()).await;
        super::no_subscribers_queues_gc::execute(app.as_ref(), topic.as_ref()).await;
    }
}
