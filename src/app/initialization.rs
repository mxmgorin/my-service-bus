use std::{sync::Arc, time::Duration};

use my_service_bus_shared::queue_with_intervals::QueueWithIntervals;

use crate::{
    topics::{Topic, TopicSnapshot},
    utils::StopWatch,
};

use super::AppContext;

pub async fn init(app: Arc<AppContext>) {
    let mut sw = StopWatch::new();
    sw.start();

    let topics_and_queues = restore_topics_and_queues(app.as_ref()).await;

    println!("Loaded topics {}", topics_and_queues.len());

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

    println!("Application is initialized in {:?}", sw.duration());
}

async fn restore_topic_pages(app: Arc<AppContext>, topic: Arc<Topic>) {
    let page_id = topic.get_current_page().await;
    crate::operations::message_pages::restore_page(
        app.as_ref(),
        topic.as_ref(),
        page_id,
        "initialization",
    )
    .await
}

async fn restore_topics_and_queues(app: &AppContext) -> Vec<TopicSnapshot> {
    let mut attempt = 0;
    loop {
        attempt += 1;

        let topics_and_queues = app.topics_and_queues_repo.load().await;

        app.logs
            .add_info(
                None,
                super::logs::SystemProcess::Init,
                "restore_topics_and_queues".to_string(),
                format!("Restoring topics and queues. Attempt {}", attempt),
            )
            .await;

        if let Ok(result) = topics_and_queues {
            return result;
        }

        let err = topics_and_queues.err().unwrap();

        app.logs
            .add_error(
                None,
                super::logs::SystemProcess::Init,
                "restore_topics_and_queues".to_string(),
                "Can not restore topics and queues".to_string(),
                Some(format!("{:?}", err)),
            )
            .await;

        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
