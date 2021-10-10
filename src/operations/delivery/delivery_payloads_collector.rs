use super::DeliverPayloadBySubscriber;

pub struct DeliveryPayloadsCollector {
    pub subscribers: Vec<DeliverPayloadBySubscriber>,
    pub current_subscriber: Option<DeliverPayloadBySubscriber>,
}

impl DeliveryPayloadsCollector {
    pub fn new() -> Self {
        Self {
            subscribers: Vec::new(),
            current_subscriber: None,
        }
    }

    pub fn set_current(&mut self, session: DeliverPayloadBySubscriber) {
        self.complete();
        self.current_subscriber = Some(session);
    }

    pub fn complete(&mut self) -> bool {
        if self.current_subscriber.is_some() {
            let mut current_subscriber = None;
            std::mem::swap(&mut current_subscriber, &mut self.current_subscriber);

            if let Some(current_subscriber) = current_subscriber {
                if current_subscriber.messages.total_size > 0 {
                    self.subscribers.push(current_subscriber);
                    return true;
                } else {
                    return false;
                }
            }
        }

        return false;
    }
}
