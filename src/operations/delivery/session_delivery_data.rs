use std::sync::Arc;

use my_service_bus_tcp_shared::TcpContract;

use crate::{
    app::AppContext, messages_bucket::MessagesBucket, queues::subscribers::SubscriberId,
    sessions::MyServiceBusSession, topics::Topic,
};

pub struct DeliverPayloadBySubscriber {
    pub messages: MessagesBucket,
    pub session: Arc<MyServiceBusSession>,
    pub subscriber_id: SubscriberId,
}

impl DeliverPayloadBySubscriber {
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
