use std::{sync::Arc, time::Duration};

use my_service_bus_shared::page_id::{get_page_id, PageId};

use crate::{
    app::{AppContext, TEST_QUEUE},
    message_pages::MessagesPage,
    messages_bucket::{MessagesBucket, MessagesBucketPage},
    queues::QueueData,
    sessions::MyServiceBusSession,
    subscribers::{Subscriber, SubscriberId},
    topics::Topic,
};

use super::{fail_result::*, OperationFailResult};

pub async fn try_to_deliver_next_messages_for_the_queue(
    app: &AppContext,
    topic: &Topic,
    queue: &mut QueueData,
) -> Result<(), OperationFailResult> {
    loop {
        let subscriber_id = queue.subscribers.get_next_subscriber_ready_to_deliver();

        if subscriber_id.is_none() {
            return Ok(());
        }

        let (subscriber_id, session) = subscriber_id.unwrap();

        let result =
            try_to_deliver_next_messages(app, topic, queue, subscriber_id, session.as_ref())
                .await?;

        if !result {
            return Ok(());
        }
    }
}

async fn try_to_deliver_next_messages(
    app: &AppContext,
    topic: &Topic,
    queue: &mut QueueData,
    subscriber_id: SubscriberId,
    session: &MyServiceBusSession,
) -> Result<bool, OperationFailResult> {
    let subscriber =
        try_to_compile_next_messages(app, topic, queue, session, subscriber_id).await?;

    if let Some(subscriber) = subscriber {
        if let Some(messages) = &subscriber.messages_on_delivery {
            let contract = crate::tcp_contracts::tcp_contract::compile_messages_delivery_contract(
                app,
                messages,
                topic,
                subscriber.queue_id.as_str(),
                subscriber_id,
            )
            .await;

            session.send(contract).await;

            return Ok(true);
        } else {
            println!("Somehow there are no messages to deliver. Bug...");

            return Ok(false);
        }
    }

    return Ok(false);
}

async fn try_to_compile_next_messages<'t>(
    app: &AppContext,
    topic: &Topic,
    queue: &'t mut QueueData,
    session: &MyServiceBusSession,
    subscriber_id: SubscriberId,
) -> Result<Option<&'t mut Subscriber>, OperationFailResult> {
    let messages = fill_messages(app, topic, queue).await;

    if messages.pages.len() > 0 {
        let subscriber = queue
            .subscribers
            .get_by_id_mut(subscriber_id)
            .ok_or(OperationFailResult::SubscriberNotFound { id: subscriber_id })?;

        if queue.queue_id == TEST_QUEUE {
            println!(
                "Has package with {} messages. First Id: {:?}",
                messages.messages_count(),
                messages.min_id
            );
        }

        subscriber.set_messages_on_delivery(messages);

        session.set_on_delivery_flag(subscriber_id).await;
        return Ok(Some(subscriber));
    } else {
        let subscriber = queue.subscribers.get_by_id_mut(subscriber_id);
        let subscriber = into_subscriber_result_mut(subscriber, subscriber_id)?;

        subscriber.reset();

        Ok(None)
    }
}

async fn fill_messages(app: &AppContext, topic: &Topic, queue: &mut QueueData) -> MessagesBucket {
    let mut result = MessagesBucket::new();

    //BUG - I found the bug.
    while let Some(next_message) = queue.peek_next_message() {
        let page_id = get_page_id(next_message.message_id);

        loop {
            let all_messages_size = result.total_size;
            let all_messages_count = result.messages_count();

            if let Some(bucket_page) = result.get_last_page_with_id(page_id) {
                let next_message_size_result = bucket_page
                    .page
                    .get_message_size(&next_message.message_id)
                    .await;

                if let Some(next_msg_size) = next_message_size_result {
                    if all_messages_size + next_msg_size > app.max_delivery_size
                        && all_messages_count > 0
                    {
                        return result;
                    }

                    queue.dequeue_next_message();

                    bucket_page.add(
                        next_message.message_id,
                        next_message.attempt_no,
                        next_msg_size,
                    );

                    result.add_total_size(next_message.message_id, next_msg_size);

                    break;
                } else {
                    println!(
                        "Somehow we did not find message size for the Message {:?}",
                        next_message
                    );
                    break;
                }
            }

            let page = get_page(app, topic, page_id).await;
            result.add_page(MessagesBucketPage::new(page));
        }
    }

    return result;
}

async fn get_page(app: &AppContext, topic: &Topic, page_id: PageId) -> Arc<MessagesPage> {
    loop {
        let message_from_cache = topic.messages.get(page_id).await;

        if let Some(result) = message_from_cache {
            return result;
        }

        load_page_to_cache(app, topic, page_id).await;
    }
}

async fn load_page_to_cache(app: &AppContext, topic: &Topic, page_id: PageId) {
    let mut attempt_no = 0;
    loop {
        let result = app
            .messages_pages_repo
            .load_page(topic.topic_id.as_str(), page_id)
            .await;

        if let Ok(page) = result {
            topic.messages.restore_page(page).await;
            return;
        }

        //TODO - Handle Situation - if we do not have page at all - we load empty page

        let err = result.err().unwrap();

        app.logs
            .add_error(
                Some(topic.topic_id.to_string()),
                crate::app::logs::SystemProcess::Init,
                "get_page".to_string(),
                format!(
                    "Can not load page #{} from persistence storage. Attempt #{}",
                    page_id, attempt_no
                ),
                Some(format!("{:?}", err)),
            )
            .await;

        attempt_no += 1;
        tokio::time::sleep(Duration::from_secs(1)).await
    }
}
