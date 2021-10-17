use std::time::Duration;

use my_service_bus_shared::{
    messages_page::MessagesPage,
    page_id::{PageId, MESSAGES_IN_PAGE},
    MessageId, MySbMessage,
};

use crate::{app::AppContext, topics::Topic};

pub async fn init_page_to_cache(app: &AppContext, topic: &Topic, page_id: PageId) {
    let min_message_id = topic.get_min_message_id().await;

    let page_first_message_id = page_id * MESSAGES_IN_PAGE;

    match min_message_id.queue_min_message_id {
        Some(queue_min_message_id) => {
            if queue_min_message_id <= page_first_message_id {
                restore_page_to_cache(app, topic, page_id, 0, 0).await;
            } else {
                restore_page_to_cache(
                    app,
                    topic,
                    page_id,
                    queue_min_message_id,
                    min_message_id.topic_message_id,
                )
                .await;
            }
        }
        None => {
            let messages = generate_not_loaded_messages(
                page_first_message_id,
                min_message_id.topic_message_id,
            );

            let page = MessagesPage::new(page_id);
            page.restore(messages).await;
            topic.messages.restore_page(page).await;
        }
    }
}

pub async fn load_full_page_to_cache(app: &AppContext, topic: &Topic, page_id: PageId) {
    restore_page_to_cache(app, topic, page_id, 0, 0).await
}

#[inline]
fn generate_not_loaded_messages(
    from_message_id: MessageId,
    to_message_id: MessageId,
) -> Vec<MySbMessage> {
    let mut result = Vec::new();
    for id in from_message_id..to_message_id {
        result.push(MySbMessage::NotLoaded { id })
    }

    result
}

#[inline]
async fn restore_page_to_cache(
    app: &AppContext,
    topic: &Topic,
    page_id: PageId,
    from_message_id: MessageId,
    to_message_id: MessageId,
) {
    let mut attempt_no = 0;
    loop {
        let result = app
            .messages_pages_repo
            .load_page(
                topic.topic_id.as_str(),
                page_id,
                from_message_id,
                to_message_id,
            )
            .await;

        if let Ok(page) = result {
            topic.messages.restore_page(page).await;
            return;
        }

        let err = result.err().unwrap();
        match err {
            crate::persistence::PersistenceError::ZipOperationError(zip_error) => {
                app.logs
                    .add_error(
                        Some(topic.topic_id.to_string()),
                        crate::app::logs::SystemProcess::Init,
                        "get_page".to_string(),
                        format!(
                            "Can not load page #{} from persistence storage. Attempt #{}. Creating empty page",
                            page_id, attempt_no
                        ),
                        Some(format!("{:?}", zip_error)),
                    )
                    .await;

                let page = MessagesPage::new(page_id);
                topic.messages.restore_page(page).await;
                return;
            }
            _ => {
                app.logs
                    .add_error(
                        Some(topic.topic_id.to_string()),
                        crate::app::logs::SystemProcess::Init,
                        "get_page".to_string(),
                        format!(
                        "Can not load page #{} from persistence storage. Attempt #{}. Retrying...",
                        page_id, attempt_no
                    ),
                        Some(format!("{:?}", err)),
                    )
                    .await;
            }
        }

        attempt_no += 1;

        if attempt_no == 5 {
            let page = MessagesPage::new(page_id);
            topic.messages.restore_page(page).await;
            return;
        }
        tokio::time::sleep(Duration::from_secs(1)).await
    }
}
