use my_service_bus_shared::{
    protobuf_models::MessageProtobufModel, queue_with_intervals::QueueWithIntervals,
    sub_page::SubPage, MessageId,
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
                let model: MessageProtobufModel = msg.into();
                result.push(model);
            }
        }

        result
    }

    pub fn commit_persisted_messages(&mut self, topic_id: &str, ids: &[MessageId]) {
        for id in ids {
            if let Err(err) = self.messages_to_persist.remove(*id) {
                let first = ids.first();
                let last = ids.last();
                println!(
                    "Topic: {}. SubPage: {}. First:{:?}. Last:{:?}. We are trying to confirm persisted message {} - but something went wrong. Reason: {:?}",
                   topic_id,  self.sub_page.sub_page_id.value, first, last, id, err
                )
            }
        }
    }

    pub fn can_be_gced(&self) -> bool {
        self.messages_to_persist.len() == 0
    }
}
