use std::sync::Arc;

use my_service_bus_shared::{messages_bucket::MessagesBucket, messages_page::MessagesPage};

use crate::{queue_subscribers::SubscriberId, sessions::MyServiceBusSession};

pub struct DeliverPayloadBySubscriber {
    pub messages: MessagesBucket,
    pub session: Arc<MyServiceBusSession>,
    pub subscriber_id: SubscriberId,
}

impl DeliverPayloadBySubscriber {
    pub fn new(
        subscriber_id: SubscriberId,
        session: Arc<MyServiceBusSession>,
        page: Arc<MessagesPage>,
    ) -> Self {
        Self {
            subscriber_id,
            session,
            messages: MessagesBucket::new(page),
        }
    }
}
