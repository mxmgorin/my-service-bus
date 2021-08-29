use std::sync::Arc;

use my_service_bus_shared::queue_with_intervals::QueueWithIntervals;

use crate::{topics::Topic, utils::StopWatch};

use super::AppContext;

pub async fn init(app: Arc<AppContext>) {
    let mut sw = StopWatch::new();
    sw.start();
    let topics_and_queues = app.topics_and_queues_repo.load().await.unwrap();

    let topics_count = topics_and_queues.len();

    for topic_and_queues in topics_and_queues {
        let topic = app
            .topic_list
            .restore(
                topic_and_queues.topic_id.as_str(),
                topic_and_queues.message_id,
            )
            .await;

        for queue in topic_and_queues.queues {
            let queue_with_intervals = QueueWithIntervals::restore(queue.ranges);
            topic
                .restore_queue(
                    queue.queue_id.as_str(),
                    queue.queue_type,
                    queue_with_intervals,
                )
                .await;
        }
    }

    for topic in app.topic_list.get_all().await {
        restore_topic_pages(app.clone(), topic.clone()).await;
    }

    app.states.set_initialized();
    sw.pause();

    app.logs
        .add_info(
            None,
            super::logs::SystemProcess::Init,
            format!("Initialization is done in {:?}", sw.duration()),
            format!(
                "Application is initialized. Topics amount is: {}",
                topics_count
            ),
        )
        .await;
}

async fn restore_topic_pages(app: Arc<AppContext>, topic: Arc<Topic>) {
    let page_id = topic.get_current_page().await;
    crate::operations::message_pages::restore_page(app.as_ref(), topic.as_ref(), page_id).await
}
