use super::DeliverPayloadBySubscriber;

pub struct DeliveryPayloadsCollector {
    pub subscribers: Vec<DeliverPayloadBySubscriber>,
    pub current_subscriber: Option<DeliverPayloadBySubscriber>,
}

pub enum PayloadCollectorCompleteOperation {
    Completed,
    Canceled(DeliverPayloadBySubscriber),
}

impl DeliveryPayloadsCollector {
    pub fn new() -> Self {
        Self {
            subscribers: Vec::new(),
            current_subscriber: None,
        }
    }

    pub fn set_current(&mut self, session: DeliverPayloadBySubscriber) {
        self.current_subscriber = Some(session);
    }

    pub fn complete(&mut self) -> PayloadCollectorCompleteOperation {
        if self.current_subscriber.is_some() {
            let mut current_subscriber = None;
            std::mem::swap(&mut current_subscriber, &mut self.current_subscriber);

            if let Some(current_subscriber) = current_subscriber {
                if current_subscriber.messages.messages_count() > 0 {
                    self.subscribers.push(current_subscriber);
                    return PayloadCollectorCompleteOperation::Completed;
                } else {
                    return PayloadCollectorCompleteOperation::Canceled(current_subscriber);
                }
            }
        }

        panic!("Can not complete collection of Payload. No current subscriber");
    }
}
