use std::sync::Arc;

use my_service_bus_shared::page_id::{get_page_id, PageId};
use my_service_bus_tcp_shared::TcpContract;

use crate::{
    app::{logs::SystemProcess, AppContext},
    message_pages::{MessageSize, MessagesPage},
    messages_bucket::{MessagesBucket, MessagesBucketPage},
    queues::{NextMessage, QueueData},
    sessions::MyServiceBusSession,
    subscribers::SubscriberId,
    topics::Topic,
};

use super::OperationFailResult;

pub async fn try_to_complie_next_messages_from_the_queue(
    app: &AppContext,
    topic: &Topic,
    queue: &mut QueueData,
) -> Result<Vec<(TcpContract, Arc<MyServiceBusSession>, SubscriberId)>, OperationFailResult> {
    let mut result = Vec::new();
    while let Some(subscriber_id) = queue.subscribers.get_next_subscriber_ready_to_deliver() {
        let messages = fill_messages(app, topic, queue).await;

        if messages.pages.len() > 0 {
            if let Some(subscriber) = queue.subscribers.get_by_id_mut(subscriber_id) {
                subscriber.rented = true;
                let contract = crate::tcp::tcp_contracts::compile_messages_delivery_contract(
                    app,
                    &messages,
                    topic,
                    subscriber.queue_id.as_str(),
                    subscriber.id,
                )
                .await;

                subscriber.set_messages_on_delivery(messages);

                result.push((contract, subscriber.session.clone(), subscriber_id));
            }
            //subscriber.session.set_on_delivery_flag(subscriber.id).await;
        } else {
            return Ok(result);
        }
    }

    Ok(result)
}

async fn fill_messages(app: &AppContext, topic: &Topic, queue: &mut QueueData) -> MessagesBucket {
    let mut result = MessagesBucket::new();

    while let Some(next_message) = queue.peek_next_message() {
        let page_id = get_page_id(next_message.message_id);

        let all_messages_size = result.total_size;

        if all_messages_size > app.max_delivery_size {}
        let all_messages_count = result.messages_count();

        let bucket_page = get_messages_bucket_page(&mut result, topic, page_id).await;

        let msg_size = get_message_size(app, topic, &bucket_page, &next_message, page_id).await;

        if let Some(next_msg_size) = msg_size {
            if all_messages_size + next_msg_size > app.max_delivery_size && all_messages_count > 0 {
                return result;
            }

            queue.dequeue_next_message();

            bucket_page.add(
                next_message.message_id,
                next_message.attempt_no,
                next_msg_size,
            );

            result.add_total_size(next_message.message_id, next_msg_size);

            let page = get_page(app, topic, page_id).await;
            result.add_page(MessagesBucketPage::new(page));
        }
    }

    return result;
}

#[inline]
async fn get_message_size(
    app: &AppContext,
    topic: &Topic,
    msg_bucket_page: &MessagesBucketPage,
    next_message: &NextMessage,
    page_id: PageId,
) -> Option<usize> {
    let first_time =
        get_message_size_first_time(app, topic, msg_bucket_page, next_message, page_id).await;

    if let Some(result) = first_time {
        return Some(result);
    }

    return get_message_size_second_time(msg_bucket_page, next_message).await;
}

async fn get_message_size_first_time(
    app: &AppContext,
    topic: &Topic,
    bucket_page: &MessagesBucketPage,
    next_message: &NextMessage,
    page_id: PageId,
) -> Option<usize> {
    let next_message_size_result = bucket_page
        .page
        .get_message_size(&next_message.message_id)
        .await;

    match next_message_size_result {
        MessageSize::MessageIsReady(next_msg_size) => {
            return Some(next_msg_size);
        }
        MessageSize::NotLoaded => {
            super::message_pages::restore_page(app, topic, page_id).await;
            return None;
        }
        MessageSize::CanNotBeLoaded => {
            app.logs
                .add_error(
                    Some(topic.topic_id.to_string()),
                    SystemProcess::DeliveryOperation,
                    "fill_messages".to_string(),
                    "Message can not be loaded. Skipping it".to_string(),
                    Some(format!("MessageId: {}", next_message.message_id)),
                )
                .await;

            return None;
        }
    }
}

async fn get_message_size_second_time(
    bucket_page: &MessagesBucketPage,
    next_message: &NextMessage,
) -> Option<usize> {
    let next_message_size_result = bucket_page
        .page
        .get_message_size(&next_message.message_id)
        .await;

    match next_message_size_result {
        MessageSize::MessageIsReady(next_msg_size) => {
            return Some(next_msg_size);
        }
        _ => return None,
    }
}

async fn get_messages_bucket_page<'t>(
    messages_bucket: &'t mut MessagesBucket,
    topic: &Topic,
    page_id: PageId,
) -> &'t mut MessagesBucketPage {
    if let Some(last_page_id) = messages_bucket.last_page_id {
        if last_page_id == page_id {
            return messages_bucket.pages.last_mut().unwrap();
        }
    }

    let page = topic.messages.get(page_id).await.unwrap(); //TODO - Remove unwrap

    let page = MessagesBucketPage::new(page);

    messages_bucket.add_page(page);

    return messages_bucket.pages.last_mut().unwrap();
}

async fn get_page(app: &AppContext, topic: &Topic, page_id: PageId) -> Arc<MessagesPage> {
    loop {
        let message_from_cache = topic.messages.get(page_id).await;

        if let Some(result) = message_from_cache {
            return result;
        }

        super::load_page_to_cache::do_it(app, topic, page_id).await;
    }
}
