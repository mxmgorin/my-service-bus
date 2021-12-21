use std::{collections::HashMap, sync::Arc, time::Duration};

use my_service_bus_shared::{page_id::PageId, queue_with_intervals::QueueWithIntervals};

use crate::{
    app::AppContext,
    topics::{Topic, TopicSnapshot},
};

pub async fn start(app: Arc<AppContext>) {
    let duration = Duration::from_secs(1);
    while !app.states.is_initialized() {
        tokio::time::sleep(duration).await;
    }

    app.logs.add_info(
        None,
        crate::app::logs::SystemProcess::Timer,
        "Persist timer".to_string(),
        "Started".to_string(),
    );

    while !app.states.app_is_shutted_down() {
        let persist_handle = tokio::task::spawn(persist_tick(app.clone()));

        if let Err(err) = persist_handle.await {
            app.logs.add_error(
                None,
                crate::app::logs::SystemProcess::Timer,
                "Persist tick".to_string(),
                "Error during doing Persist one second timer iteration".to_string(),
                Some(format!("{:?}", err)),
            );
        }

        tokio::time::sleep(duration).await
    }
}

async fn persist_tick(app: Arc<AppContext>) {
    save_topics_and_queues(app.clone()).await;

    for topic in app.topic_list.get_all().await {
        save_messages_for_topic(app.clone(), topic.clone()).await;
    }
}

pub async fn save_topics_and_queues(app: Arc<AppContext>) {
    let mut topics_snapshots = Vec::new();

    let topics = app.topic_list.get_all().await;

    for topic in topics {
        save_messages_for_topic(app.clone(), topic.clone()).await;

        {
            let topic_data = topic.data.lock().await;

            let topic_snapshot = TopicSnapshot {
                message_id: topic_data.message_id,
                topic_id: topic_data.topic_id.to_string(),
                queues: topic_data.queues.get_snapshot_to_persist(),
            };

            topics_snapshots.push(topic_snapshot);
        }
    }

    let result = app.topics_and_queues_repo.save(topics_snapshots).await;

    if let Err(err) = result {
        app.logs.add_error(
            None,
            crate::app::logs::SystemProcess::TcpSocket,
            "persist::sync_topics_and_queues".to_string(),
            "Failed to save topics and queues snapshot".to_string(),
            Some(format!("{:?}", err)),
        );
    }
}

pub async fn save_messages_for_topic(app: Arc<AppContext>, topic: Arc<Topic>) {
    let mut messages_to_persist_by_page = None;

    {
        let topic_data = topic.data.lock().await;

        while let Some((page_id, messages)) = topic_data.pages.get_messages_to_persist() {
            if messages_to_persist_by_page.is_none() {
                messages_to_persist_by_page = Some(HashMap::new());
            }

            let hash_map = messages_to_persist_by_page.as_mut().unwrap();

            hash_map.insert(page_id, messages);
        }
    }

    if messages_to_persist_by_page.is_none() {
        return;
    }

    let messages_to_persist_by_page = messages_to_persist_by_page.unwrap();

    for (page_id, messages_to_persist) in messages_to_persist_by_page {
        let mut msg_ids = QueueWithIntervals::new();

        for msg in &messages_to_persist {
            msg_ids.enqueue(msg.message_id);
        }

        let result = app
            .messages_pages_repo
            .save_messages(
                topic.topic_id.as_str(),
                messages_to_persist,
                app.max_delivery_size,
            )
            .await;

        if let Err(err) = result {
            app.logs.add_error(
                Some(topic.topic_id.to_string()),
                crate::app::logs::SystemProcess::Timer,
                "persist_messages".to_string(),
                format!(
                    "Can not persist messages min:{:?} max:{:?}",
                    msg_ids.get_min_id(),
                    msg_ids.get_max_id()
                ),
                Some(format!("{:?}", err)),
            );
        } else {
            messages_are_persisted_ok(topic.as_ref(), page_id, msg_ids).await;
        }
    }
}

async fn messages_are_persisted_ok(topic: &Topic, page_id: PageId, messages: QueueWithIntervals) {
    let mut topic_data = topic.data.lock().await;
    topic_data.pages.persisted(page_id, messages);
}
