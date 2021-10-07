use std::sync::Arc;

use my_service_bus_shared::page_id::{get_page_id, PageId};
use my_service_bus_tcp_shared::TcpContract;

use crate::{
    app::AppContext,
    message_pages::{MessageSize, MessagesPage},
    messages_bucket::MessagesBucket,
    queues::{NextMessage, QueueData, TopicQueue},
    sessions::MyServiceBusSession,
    subscribers::SubscriberId,
    topics::Topic,
};

use super::OperationFailResult;

pub struct DeliveryMessageData {
    pub sessions: Vec<DeliverMessagesToSession>,
    pub current_session: Option<DeliverMessagesToSession>,
}

impl DeliveryMessageData {
    pub fn new() -> Self {
        Self {
            sessions: Vec::new(),
            current_session: None,
        }
    }

    pub fn set_current(&mut self, session: DeliverMessagesToSession) {
        self.complete();
        self.current_session = Some(session);
    }

    pub fn complete(&mut self) {
        if self.current_session.is_some() {
            let mut current_session = None;
            std::mem::swap(&mut current_session, &mut self.current_session);
            self.sessions.push(current_session.unwrap());
        }
    }
}

pub struct DeliverMessagesToSession {
    pub messages: MessagesBucket,
    pub session: Arc<MyServiceBusSession>,
    pub subscriber_id: SubscriberId,
}

impl DeliverMessagesToSession {
    pub fn new(subscriber_id: SubscriberId, session: Arc<MyServiceBusSession>) -> Self {
        Self {
            subscriber_id,
            session,
            messages: MessagesBucket::new(),
        }
    }
    pub async fn compile_tcp_packet(
        &self,
        process_id: i64,
        app: &AppContext,
        topic: &Topic,
        queue_id: &str,
    ) -> TcpContract {
        crate::tcp::tcp_contracts::compile_messages_delivery_contract(
            process_id,
            app,
            &self.messages,
            topic,
            queue_id,
            self.subscriber_id,
        )
        .await
    }
}

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
    tokio::spawn(deliver_to_queue_swapned(process_id, app, topic, queue));
}

async fn deliver_to_queue_swapned(
    process_id: i64,
    app: Arc<AppContext>,
    topic: Arc<Topic>,
    queue: Arc<TopicQueue>,
) -> Result<(), OperationFailResult> {
    let mut delivery_data = DeliveryMessageData::new();

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
                app.as_ref(),
                topic.as_ref(),
                &mut queue_write_access,
                &mut delivery_data,
            )
            .await;

            queue_write_access.update_metrics(&queue.metrics).await;
            app.exit_lock(process_id).await;
        }

        match compile_result {
            CompileResult::Completed => {
                break;
            }
            CompileResult::LoadPage(page_id) => {
                println!(
                    "We do not have page {} for the topic {} to delivery messages. Restoring",
                    page_id, topic.topic_id
                );
                crate::operations::load_page_to_cache::do_it(app.as_ref(), topic.as_ref(), page_id)
                    .await;
            }
        }
    }

    delivery_data.complete();
    for delivery_data in delivery_data.sessions {
        let tcp_contract = delivery_data
            .compile_tcp_packet(
                process_id,
                app.as_ref(),
                topic.as_ref(),
                queue.queue_id.as_str(),
            )
            .await;

        delivery_data
            .session
            .send_and_set_on_delivery(process_id, tcp_contract, delivery_data.subscriber_id)
            .await;

        todo!("Do not forget set messages to session")
    }

    Ok(())
}

async fn try_to_complie_next_messages_from_the_queue(
    app: &AppContext,
    topic: &Topic,
    queue: &mut QueueData,
    delivery_data: &mut DeliveryMessageData,
) -> CompileResult {
    loop {
        if delivery_data.current_session.is_none() {
            if let Some(subscriber) = queue.subscribers.get_next_subscriber_ready_to_deliver() {
                delivery_data.set_current(DeliverMessagesToSession::new(
                    subscriber.id,
                    subscriber.session.clone(),
                ))
            } else {
                return CompileResult::Completed;
            }
        }

        let session = delivery_data.current_session.as_mut().unwrap();

        let result = fill_messages(app, topic, queue, &mut session.messages).await;

        match result {
            FillMessagesResult::Complete => {
                queue.subscribers.set_as_rented(session.subscriber_id);
                delivery_data.complete();
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
            super::message_pages::restore_page(app, topic, page_id, "get_message_size_first_time")
                .await;
            return None;
        }
        MessageSize::CanNotBeLoaded => {
            return None;
        }
    }
}
