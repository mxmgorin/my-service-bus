use my_service_bus_shared::{
    protobuf_models::MessageProtobufModel, queue_with_intervals::QueueWithIntervals,
    sub_page::SubPage, MessageId, MySbMessage,
};

pub struct SubPageData {
    pub sub_page: SubPage,
    pub messages_to_persist: QueueWithIntervals,
}

impl SubPageData {
    pub fn new(sub_page: SubPage) -> Self {
        Self {
            sub_page,
            messages_to_persist: QueueWithIntervals::new(),
        }
    }

    pub fn compile_messages_to_persist(&self) -> Vec<MessageProtobufModel> {
        let mut result = Vec::new();

        for message_id in &self.messages_to_persist {
            if let Some(msg) = self.sub_page.get_message(message_id) {
                match msg {
                    MySbMessage::Loaded(msg) => {
                        let model: MessageProtobufModel = msg.into();
                        result.push(model);
                    }
                    MySbMessage::Missing { id: _ } => {}
                }
            }
        }

        result
    }

    //TODO - Unit Test It
    pub fn commit_persisted_messages(&mut self, ids: &[MessageId]) {
        for id in ids {
            self.messages_to_persist.remove(*id);
        }
    }
}
