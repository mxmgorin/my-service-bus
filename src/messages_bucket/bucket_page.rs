use std::{collections::HashMap, sync::Arc};

use my_service_bus_shared::{queue_with_intervals::QueueWithIntervals, MessageId};

use crate::message_pages::MessagesPage;

use super::MessageToSendModel;

//TODO - UnitTest It
pub struct MessagesBucketPage {
    pub page: Arc<MessagesPage>,
    pub messages: HashMap<MessageId, MessageToSendModel>,
    pub messages_size: usize,
    pub ids: QueueWithIntervals,
}

impl MessagesBucketPage {
    pub fn new(page: Arc<MessagesPage>) -> Self {
        Self {
            page,
            messages: HashMap::new(),
            messages_size: 0,
            ids: QueueWithIntervals::new(),
        }
    }

    pub fn add(&mut self, msg_id: MessageId, attempt_no: i32, msg_size: usize) {
        let message = MessageToSendModel {
            msg_id,
            attempt_no,
            msg_size,
        };

        self.messages.insert(message.msg_id, message);
        self.messages_size += msg_size;

        self.ids.enqueue(msg_id);
    }

    pub fn messages_count(&self) -> usize {
        return self.messages.len();
    }

    pub fn remove(&mut self, message_id: MessageId) -> Option<MessageToSendModel> {
        let removed_message = self.messages.remove(&message_id)?;

        self.messages_size -= removed_message.msg_size;

        self.ids.remove(message_id);

        return Some(removed_message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_messages_count_and_messages_size() {
        let page = MessagesPage::new(0);
        let page = Arc::new(page);

        let mut mbp = MessagesBucketPage::new(page);

        mbp.add(5, 0, 3);
        mbp.add(6, 0, 2);

        assert_eq!(2, mbp.messages_count());
        assert_eq!(5, mbp.messages_size);
    }

    #[test]
    fn test_messages_count_and_messages_size_after_remove_item() {
        let page = MessagesPage::new(0);
        let page = Arc::new(page);

        let mut mbp = MessagesBucketPage::new(page);

        mbp.add(5, 0, 3);
        mbp.add(6, 0, 2);

        mbp.remove(5);

        assert_eq!(1, mbp.messages_count());
        assert_eq!(2, mbp.messages_size);
    }
}
