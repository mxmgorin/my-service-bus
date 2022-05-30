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
        let topics = tokio::task::spawn(tick_topics(app.clone()));

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

async fn tick_topics(app: Arc<AppContext>) {
    app.topic_list.one_second_tick().await;
    app.sessions.one_second_tick().await;

    let mut permanent_queues_without_subscribers = 0;
    let mut topics_without_queues = 0;

    for topic in app.topic_list.get_all().await {
        let topic_data = topic.get_access("tick_topics").await;

        let persist_queue_size = topic_data.pages.get_persist_queue_size();

        app.prometheus
            .update_persist_queue_size(topic.topic_id.as_str(), persist_queue_size);

        let mut queues_count = 0;

        for queue in topic_data.queues.get_all() {
            let queue_size = queue.get_queue_size();
            queues_count += 1;
            app.prometheus.update_topic_queue_size(
                topic.topic_id.as_str(),
                queue.queue_id.as_str(),
                queue_size,
            );

            let is_permanent = match &queue.queue_type {
                my_service_bus_shared::queue::TopicQueueType::Permanent => true,
                my_service_bus_shared::queue::TopicQueueType::DeleteOnDisconnect => false,
                my_service_bus_shared::queue::TopicQueueType::PermanentWithSingleConnection => true,
            };

            if is_permanent && queue.subscribers.get_amount() == 0 {
                permanent_queues_without_subscribers += 1;
            }
        }

        if queues_count == 0 {
            topics_without_queues += 1;
        }
    }

    app.prometheus
        .update_permanent_queues_without_subscribers(permanent_queues_without_subscribers);

    app.prometheus
        .update_topics_without_queues(topics_without_queues);
}
