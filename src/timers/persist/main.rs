use std::{sync::Arc, time::Duration};

use crate::{app::AppContext, topics::Topic};

pub async fn start(app: Arc<AppContext>) {
    let duration = Duration::from_secs(1);
    while !app.states.is_initialized() {
        tokio::time::sleep(duration).await;
    }

    app.logs.add_info(
        None,
        crate::app::logs::SystemProcess::Timer,
        "Persist timer".to_string(),
        "Started".to_string(),
    );

    while !app.states.app_is_shutted_down() {
        let persist_handle = tokio::task::spawn(persist_tick(app.clone()));

        if let Err(err) = persist_handle.await {
            app.logs.add_error(
                None,
                crate::app::logs::SystemProcess::Timer,
                "Persist tick".to_string(),
                "Error during doing Persist one second timer iteration".to_string(),
                Some(format!("{:?}", err)),
            );
        }

        tokio::time::sleep(duration).await
    }
}

async fn persist_tick(app: Arc<AppContext>) {
    let topics = super::persist_topics_and_queues::save(app.clone()).await;

    for topic in topics {
        save_messages_for_topic(app.clone(), topic.clone()).await;
    }
}

pub async fn save_messages_for_topic(app: Arc<AppContext>, topic: Arc<Topic>) {
    let messages_to_persist_by_page =
        super::persist_messages::get_messages_to_persist_by_page(topic.as_ref()).await;

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
            super::persist_messages::commit_persist_result(topic.as_ref(), page_id, false).await;
        } else {
            super::persist_messages::commit_persist_result(topic.as_ref(), page_id, true).await;
        }
    }
}
