use my_service_bus_shared::{page_id::PageId, MessageId};
use tokio::sync::RwLock;

use crate::{
    date_time::AtomicDateTime,
    messages::{MySbMessage, MySbMessageContent},
};

use super::MessagesPageData;

pub enum MessageSize {
    MessageIsReady(usize),
    NotLoaded,
    CanNotBeLoaded,
}

pub struct MessagesPage {
    pub data: RwLock<MessagesPageData>,
    pub page_id: PageId,
    pub last_accessed: AtomicDateTime,
}

impl MessagesPage {
    pub fn new(page_id: PageId) -> MessagesPage {
        MessagesPage {
            data: RwLock::new(MessagesPageData::new()),
            page_id,
            last_accessed: AtomicDateTime::utc_now(),
        }
    }

    pub async fn new_messages(&self, msgs: Vec<MySbMessageContent>) {
        let mut write_access = self.data.write().await;
        write_access.new_messages(msgs);
    }

    pub async fn restore(&self, msgs: Vec<MySbMessage>) {
        let mut write_access = self.data.write().await;
        write_access.restore(msgs);
    }

    pub async fn get_message_size(&self, message_id: &MessageId) -> MessageSize {
        let read_access = self.data.read().await;
        let msg = read_access.messages.get(message_id);

        if msg.is_none() {
            //TODO - Double check
            return MessageSize::NotLoaded;
        }

        match msg.unwrap() {
            MySbMessage::Loaded(msg) => MessageSize::MessageIsReady(msg.content.len()),
            MySbMessage::CanNotBeLoaded { id: _, err: _ } => MessageSize::CanNotBeLoaded,
            MySbMessage::NotLoaded { id: _ } => MessageSize::NotLoaded,
        }
    }
}
