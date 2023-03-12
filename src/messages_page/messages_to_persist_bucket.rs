use my_service_bus_abstractions::MessageId;
use my_service_bus_shared::{protobuf_models::MessageProtobufModel, MessageId};

pub struct MessagesToPersistBucket {
    messages_to_persist: Option<Vec<MessageProtobufModel>>,
    pub id: usize,
    pub first_message_id: MessageId,
}

impl MessagesToPersistBucket {
    pub fn new(id: usize, messages_to_persist: Vec<MessageProtobufModel>) -> Self {
        let first_message_id = messages_to_persist[0].message_id;

        Self {
            messages_to_persist: Some(messages_to_persist),
            first_message_id,
            id,
        }
    }

    pub fn get(&mut self) -> Vec<MessageProtobufModel> {
        let mut result = None;

        std::mem::swap(&mut result, &mut self.messages_to_persist);
        result.unwrap()
    }
}
