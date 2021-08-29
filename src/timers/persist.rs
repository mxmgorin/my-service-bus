use std::{sync::Arc, time::Duration};

use my_service_bus_shared::queue_with_intervals::QueueWithIntervals;

use crate::{
    app::{logs::Logs, AppContext},
    bcl_proto::BclDateTime,
    date_time::MyDateTime,
    message_pages::MessagesPage,
    messages::MySbMessage,
    persistence::protobuf_models::MessageProtobufModel,
    topics::Topic,
};

pub async fn start(app: Arc<AppContext>) {
    let duration = Duration::from_secs(1);
    while !app.states.is_initialized() {
        tokio::time::sleep(duration).await;
    }

    app.logs
        .add_info(
            None,
            crate::app::logs::SystemProcess::Timer,
            "Persist timer".to_string(),
            "Started".to_string(),
        )
        .await;

    while !app.states.app_is_shutted_down() {
        let persist_handle = tokio::task::spawn(persist_tick(app.clone()));

        if let Err(err) = persist_handle.await {
            app.logs
                .add_error(
                    None,
                    crate::app::logs::SystemProcess::Timer,
                    "Persist tick".to_string(),
                    "Error during doing Persist one second timer iteration".to_string(),
                    Some(format!("{:?}", err)),
                )
                .await;
        }

        tokio::time::sleep(duration).await
    }
}

async fn persist_tick(app: Arc<AppContext>) {
    sync_topics_and_queues(app.clone()).await;
    persist_messages(app.as_ref()).await;
}

pub async fn sync_topics_and_queues(app: Arc<AppContext>) {
    let snapshot = app.topic_list.get_snapshot().await;
    let result = app.topics_and_queues_repo.save(snapshot).await;

    if let Err(err) = result {
        app.logs
            .add_error(
                None,
                crate::app::logs::SystemProcess::TcpSocket,
                "persist::sync_topics_and_queues".to_string(),
                "Failed to save topics and queues snapshot".to_string(),
                Some(format!("{:?}", err)),
            )
            .await;
    }
}

async fn persist_messages(app: &AppContext) {
    let topics = app.topic_list.get_all().await;

    for topic in topics {
        persit_messages_for_topic(app, topic.as_ref()).await;
    }
}

pub async fn persit_messages_for_topic(app: &AppContext, topic: &Topic) {
    let pages = topic.messages.get_pages().await;

    for page in pages {
        let messages_to_persist =
            get_messages_to_persist(page.as_ref(), app.logs.as_ref(), topic.topic_id.as_str())
                .await;

        if let Some(messages) = messages_to_persist {
            let result = app
                .messages_pages_repo
                .save_messages(topic.topic_id.as_str(), messages.0, app.max_delivery_size)
                .await;

            if let Err(err) = result {
                app.logs
                    .add_error(
                        Some(topic.topic_id.to_string()),
                        crate::app::logs::SystemProcess::Timer,
                        "persist_messages".to_string(),
                        format!(
                            "Can not persist messages min:{:?} max:{:?}",
                            messages.1.get_min_id(),
                            messages.1.get_max_id()
                        ),
                        Some(format!("{:?}", err)),
                    )
                    .await;

                put_messages_to_persist_back(page.as_ref(), &messages.1).await;
            } else {
                messages_are_persisted_ok(page.as_ref()).await;
            }
        }
    }
}

async fn get_messages_to_persist(
    page: &MessagesPage,
    logs: &Logs,
    topic_id: &str,
) -> Option<(Vec<MessageProtobufModel>, QueueWithIntervals)> {
    {
        let read_access = page.data.read().await;

        if read_access.to_be_persisted.len() == 0 {
            return None;
        }
    }

    let mut result = Vec::new();

    let mut write_access = page.data.write().await;

    let queue_result = write_access.to_be_persisted.clone();

    while let Some(msg_id) = write_access.to_be_persisted.dequeue() {
        let msg = write_access.messages.get(&msg_id);

        if let Some(my_sb_message) = msg {
            if let MySbMessage::Loaded(msg) = my_sb_message {
                let create = BclDateTime::from(msg.time);

                result.push(MessageProtobufModel {
                    created: Some(create),
                    data: msg.content.to_vec(),
                    message_id: msg_id,
                    metadata: Vec::new(),
                });
            } else {
                logs.add_error(
                    Some(topic_id.to_string()),
                    crate::app::logs::SystemProcess::Persistence,
                    format!("Getting messages to persist for page: {}", page.page_id),
                    "Somehow we do not have loaded message but we have to persist it. Bug..."
                        .to_string(),
                    None,
                )
                .await;

                //TODO - Make sure that we do not GC Messages before we persist them.
            }
        }
    }

    if queue_result.len() == 0 {
        return None;
    }

    write_access.is_being_persisted = true;

    Some((result, queue_result))
}

async fn put_messages_to_persist_back(page: &MessagesPage, msgs: &QueueWithIntervals) {
    let mut write_access = page.data.write().await;

    for msg_id in msgs.clone() {
        write_access.to_be_persisted.enqueue(msg_id);
    }

    write_access.is_being_persisted = false;
}

async fn messages_are_persisted_ok(page: &MessagesPage) {
    let mut write_access = page.data.write().await;
    write_access.is_being_persisted = false;
}

impl From<MyDateTime> for BclDateTime {
    fn from(src: MyDateTime) -> Self {
        BclDateTime {
            value: src.micros * 20,
            scale: crate::bcl_proto::bcl_date_time_utils::SCALE_TICKS,
            kind: 0,
        }
    }
}
