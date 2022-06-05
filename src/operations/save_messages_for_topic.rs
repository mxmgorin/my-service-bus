use std::sync::Arc;

use my_service_bus_shared::{sub_page::SubPageId, MessageId};

use crate::{app::AppContext, topics::Topic};

pub async fn save_messages_for_topic(app: &Arc<AppContext>, topic: &Arc<Topic>) {
    let messages_to_persist = super::get_next_messages_to_persist(topic.as_ref()).await;

    if messages_to_persist.is_none() {
        return;
    }

    let (sub_page_id, messages_to_persist) = messages_to_persist.unwrap();

    let mut messages_ids = Vec::new();

    for msg in &messages_to_persist {
        messages_ids.push(msg.message_id);
    }

    let result = app
        .messages_pages_repo
        .save_messages(topic.topic_id.as_str(), messages_to_persist)
        .await;

    if let Err(err) = result {
        app.logs.add_error(
            Some(topic.topic_id.to_string()),
            crate::app::logs::SystemProcess::Timer,
            "persist_messages".to_string(),
            format!("Can not persist messages from id:{:?}", messages_ids[0]),
            Some(format!("{:?}", err)),
        );
    } else {
        commit_persisted(topic.as_ref(), sub_page_id, &messages_ids).await;
    }
}

async fn commit_persisted(topic: &Topic, sub_page_id: SubPageId, messages: &[MessageId]) {
    let mut topic_data = topic.get_access().await;
    topic_data
        .pages
        .commit_persisted_messages(sub_page_id, messages);
}
