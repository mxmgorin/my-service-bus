use std::{sync::Arc, time::Duration};

use crate::app::AppContext;

pub async fn start(app: Arc<AppContext>) {
    let duration = Duration::from_secs(1);

    while !app.states.is_initialized() {
        tokio::time::sleep(duration).await;
    }

    app.logs.add_info(
        None,
        crate::app::logs::SystemProcess::Timer,
        "Metrics timer".to_string(),
        "Started".to_string(),
    );

    while !app.states.app_is_shutted_down() {
        let sessions = tokio::task::spawn(tick_sessions(app.clone()));
        let topics = tokio::task::spawn(tick_topics(app.clone()));

        if let Err(err) = sessions.await {
            app.logs.add_error(
                None,
                crate::app::logs::SystemProcess::Timer,
                "Sessions one second".to_string(),
                "Error during doing Sessions one second timer iteration".to_string(),
                Some(format!("{:?}", err)),
            );
        }

        if let Err(err) = topics.await {
            app.logs.add_error(
                None,
                crate::app::logs::SystemProcess::Timer,
                "Topics one second".to_string(),
                "Error during doing Topics one second timer iteration".to_string(),
                Some(format!("{:?}", err)),
            );
        }

        tokio::time::sleep(duration).await
    }
}

async fn tick_sessions(app: Arc<AppContext>) {
    app.sessions.one_second_tick().await;
}

async fn tick_topics(app: Arc<AppContext>) {
    app.topic_list.one_second_tick().await;

    for topic in app.topic_list.get_all().await {
        let topic_data = topic.data.lock().await;

        let persist_queue_size = topic_data.pages.get_persist_queue_size();

        app.prometheus
            .update_persist_queue_size(topic.topic_id.as_str(), persist_queue_size);

        for queue in topic_data.queues.get_all() {
            let queue_size = queue.get_queue_size();
            app.prometheus.update_topic_queue_size(
                topic.topic_id.as_str(),
                queue.queue_id.as_str(),
                queue_size,
            );
        }
    }
}
