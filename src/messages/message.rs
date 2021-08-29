use my_service_bus_shared::MessageId;

use crate::date_time::MyDateTime;

#[derive(Debug)]
pub struct MySbMessageContent {
    pub id: MessageId,
    pub content: Vec<u8>,
    pub time: MyDateTime,
}

impl MySbMessageContent {
    pub fn new(id: MessageId, content: Vec<u8>, time: MyDateTime) -> Self {
        Self {
            id,
            content: content,
            time,
        }
    }
}

#[derive(Debug)]
pub enum MySbMessage {
    Loaded(MySbMessageContent),
    CanNotBeLoaded { id: MessageId, err: String },
    // NotLoaded { id: MessageId }, //TODO - Make the cases where we do GC used Messages
}

impl MySbMessage {
    pub fn content_size(&self) -> usize {
        match self {
            MySbMessage::Loaded(msg) => msg.content.len(),
            MySbMessage::CanNotBeLoaded { id: _, err: _ } => 0,
            // MySbMessage::NotLoaded { id: _ } => 0,
        }
    }

    pub fn get_id(&self) -> MessageId {
        match self {
            MySbMessage::Loaded(msg) => msg.id,
            MySbMessage::CanNotBeLoaded { id, err: _ } => *id,
            //MySbMessage::NotLoaded { id } => *id,
        }
    }
}
