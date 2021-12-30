use std::collections::HashMap;

use my_service_bus_shared::{page_id::PageId, protobuf_models::MessageProtobufModel};

use crate::topics::{Topic, TopicData};

pub async fn get_messages_to_persist_by_page(
    topic: &Topic,
) -> Option<HashMap<PageId, Vec<MessageProtobufModel>>> {
    let mut topic_data = topic.get_access("get_messages_to_persist_by_page").await;
    return get_messages_to_persist(&mut topic_data);
}

fn get_messages_to_persist(
    topic_data: &mut TopicData,
) -> Option<HashMap<PageId, Vec<MessageProtobufModel>>> {
    let mut messages_to_persist_by_page = None;

    while let Some((page_id, messages)) = topic_data.pages.get_messages_to_persist() {
        if messages_to_persist_by_page.is_none() {
            messages_to_persist_by_page = Some(HashMap::new());
        }

        messages_to_persist_by_page
            .as_mut()
            .unwrap()
            .insert(page_id, messages);
    }

    messages_to_persist_by_page
}

pub async fn commit_persist_result(topic: &Topic, page_id: PageId, ok: bool) {
    let mut topic_data = topic.get_access("commit_persist_result").await;

    if ok {
        topic_data.pages.persisted(page_id);
    } else {
        topic_data.pages.not_persisted(page_id);
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_no_messages_published() {
        const TOPIC_NAME: &str = "Test";
        let mut topic_data = TopicData::new(TOPIC_NAME.to_string(), 0);

        let result = get_messages_to_persist(&mut topic_data);

        assert_eq!(true, result.is_none());
    }

    #[test]
    fn test_some_messages_are_published() {
        const TOPIC_NAME: &str = "Test";

        let mut topic_data = TopicData::new(TOPIC_NAME.to_string(), 0);

        topic_data.publish_messages(1, vec![vec![0u8, 1u8, 2u8]]);

        let result = get_messages_to_persist(&mut topic_data);

        if let Some(messages) = result {
            assert_eq!(1, messages.len());
        } else {
            assert_eq!(true, result.is_none());
        }
    }
}
