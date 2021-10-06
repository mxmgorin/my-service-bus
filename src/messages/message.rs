use my_service_bus_shared::{date_time::DateTimeAsMicroseconds, MessageId};

#[derive(Debug)]
pub struct MySbMessageContent {
    pub id: MessageId,
    pub content: Vec<u8>,
    pub time: DateTimeAsMicroseconds,
}

impl MySbMessageContent {
    pub fn new(id: MessageId, content: Vec<u8>, time: DateTimeAsMicroseconds) -> Self {
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
    NotLoaded { id: MessageId },
}

impl MySbMessage {
    pub fn content_size(&self) -> usize {
        match self {
            MySbMessage::Loaded(msg) => msg.content.len(),
            MySbMessage::CanNotBeLoaded { id: _, err: _ } => 0,
            MySbMessage::NotLoaded { id: _ } => 0,
        }
    }

    pub fn get_id(&self) -> MessageId {
        match self {
            MySbMessage::Loaded(msg) => msg.id,
            MySbMessage::CanNotBeLoaded { id, err: _ } => *id,
            MySbMessage::NotLoaded { id } => *id,
        }
    }
}
