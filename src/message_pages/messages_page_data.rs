use std::collections::HashMap;

use my_service_bus_shared::{queue_with_intervals::QueueWithIntervals, MessageId};

use crate::messages::{MySbMessage, MySbMessageContent};

pub struct MessagesPageData {
    pub to_be_persisted: QueueWithIntervals,
    pub messages: HashMap<MessageId, MySbMessage>,
    pub size: usize,
    pub is_being_persisted: bool,
}

impl MessagesPageData {
    pub fn new() -> Self {
        Self {
            messages: HashMap::new(),
            size: 0,
            to_be_persisted: QueueWithIntervals::new(),
            is_being_persisted: false,
        }
    }

    pub fn new_messages(&mut self, msgs: Vec<MySbMessageContent>) {
        for msg in msgs {
            self.size += msg.content.len();

            self.to_be_persisted.enqueue(msg.id);
            let old = self.messages.insert(msg.id, MySbMessage::Loaded(msg));

            if let Some(old) = old {
                self.size -= old.content_size();
            }
        }
    }

    pub fn restore(&mut self, msgs: Vec<MySbMessage>) {
        for msg in msgs {
            self.size += msg.content_size();

            let old = self.messages.insert(msg.get_id(), msg);

            if let Some(old) = old {
                self.size -= old.content_size();
            }
        }
    }
}
