use std::sync::Arc;

use my_service_bus_shared::date_time::DateTimeAsMicroseconds;

use crate::{messages_bucket::MessagesBucket, sessions::MyServiceBusSession};

use super::SubscriberId;

pub struct Subscriber {
    pub session: Arc<MyServiceBusSession>,
    pub id: SubscriberId,
    pub queue_id: String,
    pub rented: bool,
    pub messages_on_delivery: Option<MessagesBucket>,
    pub start_delivering: DateTimeAsMicroseconds,
    pub disconnected: bool,
}

impl Subscriber {
    pub fn new(queue_id: &str, session: Arc<MyServiceBusSession>, id: SubscriberId) -> Self {
        Subscriber {
            queue_id: queue_id.to_string(),
            session,
            id,
            rented: false,
            messages_on_delivery: None,
            start_delivering: DateTimeAsMicroseconds::now(),
            disconnected: false,
        }
    }

    pub fn set_messages_on_delivery(&mut self, messages: MessagesBucket) {
        self.start_delivering = DateTimeAsMicroseconds::now();
        self.messages_on_delivery = Some(messages);
    }

    pub fn reset(&mut self) -> Option<MessagesBucket> {
        let mut result = None;

        std::mem::swap(&mut result, &mut self.messages_on_delivery);

        self.rented = false;

        result
    }
}
