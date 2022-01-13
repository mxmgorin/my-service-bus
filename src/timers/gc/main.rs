use std::{sync::Arc, time::Duration};

use crate::app::AppContext;

pub async fn start(app: Arc<AppContext>) {
    let duration = Duration::from_secs(10);

    while !app.states.is_initialized() {
        tokio::time::sleep(duration).await;
    }

    app.logs.add_info(
        None,
        crate::app::logs::SystemProcess::Timer,
        "Timer by topic".to_string(),
        "Started".to_string(),
    );

    while !app.states.app_is_shutted_down() {
        let err = tokio::spawn(timer_tick(app.clone())).await;

        if let Err(err) = err {
            app.logs.add_fatal_error(
                crate::app::logs::SystemProcess::Timer,
                "message_pages::execute".to_string(),
                format!("{}", err),
            );
        }

        tokio::time::sleep(duration).await;
    }
}

pub async fn timer_tick(app: Arc<AppContext>) {
    for topic in app.topic_list.get_all().await {
        let mut topic_data = topic.get_access("gc.timer_tick").await;
        super::message_pages::execute(app.as_ref(), &mut topic_data);
        super::no_subscribers_queues::execute(app.as_ref(), &mut topic_data);
    }

    super::gc_http_connections::execute(app.as_ref()).await;
}
