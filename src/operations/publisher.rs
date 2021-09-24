use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use my_service_bus_shared::{
    page_id::{get_page_id, PageId},
    MessageId,
};

use crate::{
    app::AppContext, messages::MySbMessageContent, sessions::MyServiceBusSession, topics::Topic,
};

use super::OperationFailResult;

pub async fn create_topic_if_not_exists(
    process_id: i64,
    app: Arc<AppContext>,
    session: &MyServiceBusSession,
    topic_id: &str,
) -> Arc<Topic> {
    let topic = app.topic_list.add_if_not_exists(topic_id).await;
    tokio::task::spawn(crate::timers::persist::sync_topics_and_queues(app));
    session.add_publisher(process_id, topic_id).await;
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

    let topic = app.topic_list.get(topic_id).await;

    let topic = super::fail_result::into_topic_result(topic, topic_id)?;

    let messages = topic.publish_messages(messages).await;

    if persist_immediately {
        tokio::task::spawn(crate::timers::persist::sync_topics_and_queues(app.clone()));
    }

    let (msgs_by_pages, msg_ids) = split_to_pages(messages);

    topic.messages.new_messages(msgs_by_pages).await;

    let queues = topic.get_all_queues().await;

    let mut to_send = Vec::new();

    for queue in queues {
        app.enter_lock(
            process_id,
            format!("Publisher[{}/{}].publish", topic_id, queue.queue_id),
        )
        .await;

        let mut write_access = queue.data.write().await;

        write_access.enqueue_messages(msg_ids.as_slice());

        let msg_to_deliver =
            crate::operations::delivery::try_to_complie_next_messages_from_the_queue(
                process_id,
                app.as_ref(),
                topic.as_ref(),
                &mut write_access,
            )
            .await?;

        to_send.extend(msg_to_deliver);

        app.exit_lock(process_id).await;
    }

    for (tcp_contract, session, subscriber_id) in to_send {
        session
            .send_and_set_on_delivery(process_id, tcp_contract, subscriber_id)
            .await;
    }

    Ok(())
}

//TODO - UnitTest It
fn split_to_pages(
    mut messages: VecDeque<MySbMessageContent>,
) -> (HashMap<PageId, Vec<MySbMessageContent>>, Vec<MessageId>) {
    let mut result = HashMap::new();

    let mut msg_ids = Vec::new();
    for msg in messages.drain(..) {
        msg_ids.push(msg.id);
        let page_id = get_page_id(msg.id);

        if !result.contains_key(&page_id) {
            result.insert(page_id, Vec::new());
        }

        result.get_mut(&page_id).unwrap().push(msg);
    }
    (result, msg_ids)
}
