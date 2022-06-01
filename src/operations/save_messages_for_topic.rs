use std::sync::Arc;

use my_service_bus_shared::page_id::PageId;

use crate::{app::AppContext, topics::Topic};

pub async fn save_messages_for_topic(app: &Arc<AppContext>, topic: &Arc<Topic>) {
    let messages_to_persist_by_page = super::get_messages_to_persist_by_page(topic.as_ref()).await;

    if messages_to_persist_by_page.is_none() {
        return;
    }

    let messages_to_persist_by_page = messages_to_persist_by_page.unwrap();

    for (page_id, messages_to_persist) in messages_to_persist_by_page {
        let first_id = if let Some(msg) = messages_to_persist.get(0) {
            Some(msg.message_id)
        } else {
            None
        };

        let result = app
            .messages_pages_repo
            .save_messages(
                topic.topic_id.as_str(),
                messages_to_persist,
                app.max_delivery_size,
            )
            .await;

        if let Err(err) = result {
            app.logs.add_error(
                Some(topic.topic_id.to_string()),
                crate::app::logs::SystemProcess::Timer,
                "persist_messages".to_string(),
                format!("Can not persist messages from id:{:?}", first_id,),
                Some(format!("{:?}", err)),
            );
            commit_persist_result(topic.as_ref(), page_id, false).await;
        } else {
            commit_persist_result(topic.as_ref(), page_id, true).await;
        }
    }
}

pub async fn commit_persist_result(topic: &Topic, page_id: PageId, ok: bool) {
    let mut topic_data = topic.get_access("commit_persist_result").await;

    if ok {
        topic_data.pages.persisted(page_id);
    } else {
        topic_data.pages.not_persisted(page_id);
    }
}
