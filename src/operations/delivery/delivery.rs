use std::sync::Arc;

use my_service_bus_shared::{
    messages_bucket::MessagesBucket,
    messages_page::{MessageSize, MessagesPage},
    page_id::{get_page_id, PageId},
    MessageId,
};
use my_service_bus_tcp_shared::TcpContract;

use crate::{
    app::AppContext,
    operations::OperationFailResult,
    queue_subscribers::QueueSubscriber,
    queues::{NextMessage, QueueData, TopicQueue},
    topics::Topic,
};

use super::{
    DeliverPayloadBySubscriber, DeliveryPayloadsCollector, PayloadCollectorCompleteOperation,
};

pub enum CompileResult {
    Completed,
    LoadPage(PageId),
}

pub fn deliver_to_queue(
    process_id: i64,
    app: Arc<AppContext>,
    topic: Arc<Topic>,
    queue: Arc<TopicQueue>,
) {
    tokio::spawn(deliver_to_queue_spawned(process_id, app, topic, queue));
}

async fn deliver_to_queue_spawned(
    process_id: i64,
    app: Arc<AppContext>,
    topic: Arc<Topic>,
    queue: Arc<TopicQueue>,
) {
    let _lock = queue.delivery_lock.lock().await;

    let result =
        try_to_deliver_to_queue(process_id, app.as_ref(), topic.as_ref(), queue.as_ref()).await;

    if let Err(err) = &result {
        app.logs
            .add_error(
                Some(topic.topic_id.to_string()),
                crate::app::logs::SystemProcess::DeliveryOperation,
                format!("deliver_to_queue {}", queue.queue_id),
                "We cought error while it was a delivery process".to_string(),
                Some(format!("{:?}", err)),
            )
            .await
    }

    let payloads_collector = result.unwrap();

    for subscriber_data in payloads_collector.subscribers {
        let packet_version = subscriber_data
            .session
            .get_packet_version(my_service_bus_tcp_shared::tcp_message_id::NEW_MESSAGES)
            .await;

        let tcp_contract = TcpContract::compile_messages_to_deliver(
            &subscriber_data.messages,
            topic.topic_id.as_str(),
            queue.queue_id.as_str(),
            subscriber_data.subscriber_id,
            packet_version,
        )
        .await;

        let send_packet;

        {
            let mut queue_write_access = queue
                .get_write_access(
                    process_id,
                    format!(
                        "deliver_to_queue_spawned[{}/{}]",
                        queue.topic_id, queue.queue_id
                    ),
                    app.as_ref(),
                )
                .await;

            let result = queue_write_access
                .data
                .subscribers
                .set_messages_on_delivery(subscriber_data.subscriber_id, subscriber_data.messages);

            if let Some(messages) = result {
                println!(
                    "Could not find subscriber {} for the {}/{}. Set {} messages back to the queue",
                    subscriber_data.subscriber_id,
                    topic.topic_id,
                    queue_write_access.data.queue_id,
                    messages.ids.len()
                );
                queue_write_access.data.enqueue_messages(&messages.ids);

                send_packet = false;
            } else {
                send_packet = true;
            }
        }

        if send_packet {
            crate::operations::sessions::send_package(
                app.as_ref(),
                subscriber_data.session.as_ref(),
                tcp_contract,
            )
            .await;
        }
    }
}

async fn try_to_deliver_to_queue(
    process_id: i64,
    app: &AppContext,
    topic: &Topic,
    queue: &TopicQueue,
) -> Result<DeliveryPayloadsCollector, OperationFailResult> {
    let mut payloads_collector = DeliveryPayloadsCollector::new();

    loop {
        let compile_result: CompileResult;

        {
            let mut queue_write_access = queue
                .get_write_access(
                    process_id,
                    format!("deliver_to_queue[{}/{}]", queue.topic_id, queue.queue_id),
                    app,
                )
                .await;

            compile_result = try_to_complie_next_messages_from_the_queue(
                app,
                topic,
                &mut queue_write_access.data,
                &mut payloads_collector,
            )
            .await;

            queue_write_access.data.update_metrics(&queue.metrics).await;
        }

        match compile_result {
            CompileResult::Completed => {
                return Ok(payloads_collector);
            }
            CompileResult::LoadPage(page_id) => {
                println!(
                    "We do not have page {} for the topic {} to delivery messages. Restoring",
                    page_id, topic.topic_id
                );
                crate::operations::load_page_to_cache::do_it(app, topic, page_id).await;
            }
        }
    }
}

async fn try_to_complie_next_messages_from_the_queue(
    app: &AppContext,
    topic: &Topic,
    queue: &mut QueueData,
    delivery_data: &mut DeliveryPayloadsCollector,
) -> CompileResult {
    loop {
        if delivery_data.current_subscriber.is_none() {
            if let Some(subscriber) = queue
                .subscribers
                .get_and_rent_next_subscriber_ready_to_deliver()
            {
                let next_message_id = queue.queue.peek();

                if next_message_id.is_none() {
                    subscriber.cancel_the_rent();
                    return CompileResult::Completed;
                }

                match get_payload_subscriber(subscriber, next_message_id.unwrap(), topic).await {
                    Ok(subscriber) => {
                        delivery_data.set_current(subscriber);
                    }
                    Err(page_id) => {
                        subscriber.cancel_the_rent();
                        return CompileResult::LoadPage(page_id);
                    }
                }
            } else {
                return CompileResult::Completed;
            }
        };

        fill_messages(
            app,
            topic,
            queue,
            &mut delivery_data.current_subscriber.as_mut().unwrap().messages,
        )
        .await;

        if let PayloadCollectorCompleteOperation::Canceled(delivery_subscriber) =
            delivery_data.complete()
        {
            queue
                .subscribers
                .get_by_id_mut(delivery_subscriber.subscriber_id)
                .unwrap()
                .cancel_the_rent();
        }
    }
}

#[inline]
async fn get_payload_subscriber(
    subscriber: &mut QueueSubscriber,
    next_message_id: MessageId,
    topic: &Topic,
) -> Result<DeliverPayloadBySubscriber, PageId> {
    let page_id = get_page_id(next_message_id);

    let page = topic.messages.get_page(page_id).await;

    if page.is_none() {
        return Err(page_id);
    }

    let result =
        DeliverPayloadBySubscriber::new(subscriber.id, subscriber.session.clone(), page.unwrap());

    return Ok(result);
}

#[inline]
async fn fill_messages(
    app: &AppContext,
    topic: &Topic,
    queue: &mut QueueData,
    messages_bucket: &mut MessagesBucket,
) {
    while let Some(next_message) = queue.peek_next_message() {
        let page_id = get_page_id(next_message.message_id);

        if messages_bucket.page.page_id != page_id {
            return;
        }

        let msg_size =
            get_message_size(app, topic, &messages_bucket.page, &next_message, page_id).await;

        if let Some(next_msg_size) = msg_size {
            if messages_bucket.messages_size + next_msg_size > app.max_delivery_size
                && messages_bucket.messages_count() > 0
            {
                break;
            }

            messages_bucket.add(
                next_message.message_id,
                next_message.attempt_no,
                next_msg_size,
            );
        }

        queue.dequeue_next_message();
    }
}

#[inline]
async fn get_message_size(
    app: &AppContext,
    topic: &Topic,
    page: &MessagesPage,
    next_message: &NextMessage,
    page_id: PageId,
) -> Option<usize> {
    let next_message_size_result = page.get_message_size(&next_message.message_id).await;

    match next_message_size_result {
        MessageSize::MessageIsReady(next_msg_size) => {
            return Some(next_msg_size);
        }
        MessageSize::NotLoaded => {
            crate::operations::message_pages::restore_page(
                app,
                topic,
                page_id,
                "get_message_size_first_time",
            )
            .await;
            return None;
        }
        MessageSize::CanNotBeLoaded => {
            return None;
        }
    }
}
