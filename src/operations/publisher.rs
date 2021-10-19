use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use my_service_bus_shared::{
    page_id::{get_page_id, PageId},
    queue_with_intervals::QueueWithIntervals,
    MySbMessageContent,
};

use crate::{app::AppContext, sessions::MyServiceBusSession, topics::Topic};

use super::OperationFailResult;

pub async fn create_topic_if_not_exists(
    app: Arc<AppContext>,
    session: Option<&MyServiceBusSession>,
    topic_id: &str,
) -> Arc<Topic> {
    let topic = app.topic_list.add_if_not_exists(topic_id).await;
    tokio::task::spawn(crate::timers::persist::sync_topics_and_queues(app));

    if let Some(session) = session {
        session.add_publisher(topic_id).await;
    }

    return topic;
}

pub async fn publish(
    process_id: i64,
    app: Arc<AppContext>,
    topic_id: &str,
    messages: Vec<Vec<u8>>,
    persist_immediately: bool,
) -> Result<(), OperationFailResult> {
    if app.states.is_shutting_down() {
        return Err(OperationFailResult::ShuttingDown);
    }

    let mut topic = app.topic_list.get(topic_id).await;

    if topic.is_none() {
        if app.auto_create_topic_on_publish {
            topic = Some(app.topic_list.add_if_not_exists(topic_id).await);
        } else {
            return Err(OperationFailResult::TopicNotFound {
                topic_id: topic_id.to_string(),
            });
        }
    }

    let topic = topic.unwrap();

    let messages = topic.publish_messages(messages).await;

    if persist_immediately {
        tokio::task::spawn(crate::timers::persist::sync_topics_and_queues(app.clone()));
    }

    let (msgs_by_pages, msg_ids) = split_to_pages(messages);

    topic.messages.new_messages(msgs_by_pages).await;

    let queues = topic.get_all_queues().await;

    for queue in queues {
        queue.enqueue_messages(&msg_ids).await;
        crate::operations::delivery::deliver_to_queue(
            process_id,
            app.clone(),
            topic.clone(),
            queue.clone(),
        );
    }

    Ok(())
}

//TODO - UnitTest It
fn split_to_pages(
    mut messages: VecDeque<MySbMessageContent>,
) -> (HashMap<PageId, Vec<MySbMessageContent>>, QueueWithIntervals) {
    let mut result = HashMap::new();

    let mut msg_ids = QueueWithIntervals::new();
    for msg in messages.drain(..) {
        msg_ids.enqueue(msg.id);
        let page_id = get_page_id(msg.id);

        if !result.contains_key(&page_id) {
            result.insert(page_id, Vec::new());
        }

        result.get_mut(&page_id).unwrap().push(msg);
    }
    (result, msg_ids)
}
