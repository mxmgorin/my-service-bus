use super::DeliverPayloadBySubscriber;

pub struct DeliveryPayloadsCollector {
    pub subscribers: Vec<DeliverPayloadBySubscriber>,
    pub current_session: Option<DeliverPayloadBySubscriber>,
}

impl DeliveryPayloadsCollector {
    pub fn new() -> Self {
        Self {
            subscribers: Vec::new(),
            current_session: None,
        }
    }

    pub fn set_current(&mut self, session: DeliverPayloadBySubscriber) {
        self.complete();
        self.current_session = Some(session);
    }

    pub fn complete(&mut self) {
        if self.current_session.is_some() {
            let mut current_session = None;
            std::mem::swap(&mut current_session, &mut self.current_session);
            self.subscribers.push(current_session.unwrap());
        }
    }
}
