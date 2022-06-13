use std::collections::HashMap;

use my_service_bus_shared::{
    protobuf_models::MessageProtobufModel, queue_with_intervals::QueueWithIntervals,
    sub_page::SubPage,
};

use super::MessagesToPersistBucket;

pub struct SubPageData {
    pub sub_page: SubPage,
    pub messages_to_persist: QueueWithIntervals,
    pub persist_id: usize,
    on_persistence: HashMap<usize, QueueWithIntervals>,
}

impl SubPageData {
    pub fn new(sub_page: SubPage) -> Self {
        Self {
            sub_page,
            messages_to_persist: QueueWithIntervals::new(),
            persist_id: 0,
            on_persistence: HashMap::new(),
        }
    }

    pub fn compile_messages_to_persist(&mut self) -> MessagesToPersistBucket {
        let mut messages_to_persist = Vec::new();
        let mut ids = QueueWithIntervals::new();

        for message_id in &self.messages_to_persist {
            if let Some(msg) = self.sub_page.get_message(message_id) {
                let model: MessageProtobufModel = msg.into();
                messages_to_persist.push(model);
            }

            ids.enqueue(message_id);
        }

        let persist_id = self.persist_id;
        self.persist_id += 1;

        self.on_persistence.insert(persist_id, ids);
        MessagesToPersistBucket::new(persist_id, messages_to_persist)
    }

    pub fn commit_persisted_messages(
        &mut self,
        topic_id: &str,
        messages_ot_persist: &MessagesToPersistBucket,
    ) {
        if let Some(ids) = self.on_persistence.remove(&messages_ot_persist.id) {
            for id in &ids {
                if let Err(err) = self.messages_to_persist.remove(id) {
                    println!(
                        "Topic: {}. SubPage: {}. We are trying to confirm persisted message {} - but something went wrong. Reason: {:?}",
                       topic_id,  self.sub_page.sub_page_id.value, id, err
                    )
                }
            }
        }
    }

    pub fn can_be_gced(&self) -> bool {
        self.messages_to_persist.len() == 0
    }
}
