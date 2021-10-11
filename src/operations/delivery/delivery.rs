use std::sync::Arc;

use my_service_bus_shared::page_id::{get_page_id, PageId};

use crate::{
    app::AppContext,
    message_pages::{MessageSize, MessagesPage},
    messages_bucket::MessagesBucket,
    operations::OperationFailResult,
    queues::{NextMessage, QueueData, TopicQueue},
    topics::Topic,
};

use super::{DeliverPayloadBySubscriber, DeliveryPayloadsCollector};

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
        let tcp_contract = subscriber_data
            .compile_tcp_packet(
                process_id,
                app.as_ref(),
                topic.as_ref(),
                queue.queue_id.as_str(),
            )
            .await;

        let send_packet;

        {
            let mut queue_data = queue.data.write().await;

            let result = queue_data
                .subscribers
                .set_messages_on_delivery(subscriber_data.subscriber_id, subscriber_data.messages);

            if let Some(messages) = result {
                let msgs = messages.get_ids();
                println!(
                    "Could not find subscriber {} for the {}/{}. Set {} messages back to the queue",
                    subscriber_data.subscriber_id,
                    topic.topic_id,
                    queue_data.queue_id,
                    msgs.len()
                );
                queue_data.enqueue_messages(&msgs);

                send_packet = false;
            } else {
                send_packet = true;
            }
        }

        if send_packet {
            crate::operations::sessions::send_package(
                process_id,
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
        queue.delivery_lock.lock().await;

        app.enter_lock(
            process_id,
            format!("deliver_to_queue[{}/{}]", queue.topic_id, queue.queue_id),
        )
        .await;

        let compile_result: CompileResult;

        {
            let mut queue_write_access = queue.data.write().await;

            compile_result = try_to_complie_next_messages_from_the_queue(
                app,
                topic,
                &mut queue_write_access,
                &mut payloads_collector,
            )
            .await;

            queue_write_access.update_metrics(&queue.metrics).await;
            app.exit_lock(process_id).await;
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
                delivery_data.set_current(DeliverPayloadBySubscriber::new(
                    subscriber.id,
                    subscriber.session.clone(),
                ));
            } else {
                return CompileResult::Completed;
            }
        }

        let fill_messages_result;
        let subscriber_id;
        {
            let current_subscriber = delivery_data.current_subscriber.as_mut().unwrap();
            subscriber_id = current_subscriber.subscriber_id;

            fill_messages_result =
                fill_messages(app, topic, queue, &mut current_subscriber.messages).await;
        }

        match fill_messages_result {
            FillMessagesResult::Complete => {
                if !delivery_data.complete() {
                    let subscriber = queue
                        .subscribers
                        .get_by_id_mut(subscriber_id)
                        .expect(format!("Subscriber with id {} not found", subscriber_id).as_str());

                    subscriber.cancel_the_rent();
                }

                return CompileResult::Completed;
            }
            FillMessagesResult::LoadPage(page_id) => return CompileResult::LoadPage(page_id),
        }
    }
}

pub enum FillMessagesResult {
    Complete,
    LoadPage(PageId),
}

async fn fill_messages(
    app: &AppContext,
    topic: &Topic,
    queue: &mut QueueData,
    messages_bucket: &mut MessagesBucket,
) -> FillMessagesResult {
    while let Some(next_message) = queue.peek_next_message() {
        let page_id = get_page_id(next_message.message_id);

        let all_messages_size = messages_bucket.total_size;

        if all_messages_size > app.max_delivery_size {}
        let all_messages_count = messages_bucket.messages_count();

        if !messages_bucket.has_page(page_id) {
            let page = topic.messages.get(page_id).await;

            if page.is_none() {
                return FillMessagesResult::LoadPage(page_id);
            }

            messages_bucket.add_page(page.unwrap());
        }

        let bucket_page = messages_bucket.get_page(page_id);

        let msg_size =
            get_message_size(app, topic, &bucket_page.page, &next_message, page_id).await;

        if let Some(next_msg_size) = msg_size {
            if all_messages_size + next_msg_size > app.max_delivery_size && all_messages_count > 0 {
                return FillMessagesResult::Complete;
            }

            bucket_page.add(
                next_message.message_id,
                next_message.attempt_no,
                next_msg_size,
            );

            messages_bucket.add_total_size(next_message.message_id, next_msg_size);
        }

        queue.dequeue_next_message();
    }

    return FillMessagesResult::Complete;
}

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
