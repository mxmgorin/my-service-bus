use std::{collections::HashMap, time::Duration};

use my_service_bus_shared::{
    messages_page::MessagesPageRestoreSnapshot, page_id::PageId, MessageId, MySbMessageContent,
};

use crate::{app::logs::Logs, persistence::MessagesPagesRepo, topics::Topic};

pub async fn load_page<TMessagesPagesRepo: MessagesPagesRepo>(
    topic: &Topic,
    messages_pages_repo: &TMessagesPagesRepo,
    logs: Option<&Logs>,
    page_id: PageId,
    from_message_id: MessageId,
    to_message_id: MessageId,
) -> MessagesPageRestoreSnapshot {
    let messages = load_page_from_repo(
        topic,
        messages_pages_repo,
        logs,
        page_id,
        from_message_id,
        to_message_id,
    )
    .await;

    match messages {
        Some(messages) => MessagesPageRestoreSnapshot::new_with_messages(
            page_id,
            from_message_id,
            to_message_id,
            messages,
        ),
        None => MessagesPageRestoreSnapshot::new(page_id, from_message_id, to_message_id),
    }
}

#[inline]
async fn load_page_from_repo<TMessagesPagesRepo: MessagesPagesRepo>(
    topic: &Topic,
    messages_pages_repo: &TMessagesPagesRepo,
    logs: Option<&Logs>,
    page_id: PageId,
    from_message_id: MessageId,
    to_message_id: MessageId,
) -> Option<HashMap<MessageId, MySbMessageContent>> {
    let mut attempt_no = 0;
    loop {
        let result = messages_pages_repo
            .load_page(
                topic.topic_id.as_str(),
                page_id,
                from_message_id,
                to_message_id,
            )
            .await;

        if let Ok(result) = result {
            return result;
        }

        let err = result.err().unwrap();
        match err {
            crate::persistence::PersistenceError::ZipOperationError(zip_error) => {
                if let Some(logs) = logs {
                    logs
                    .add_error(
                        Some(topic.topic_id.to_string()),
                        crate::app::logs::SystemProcess::Init,
                        "get_page".to_string(),
                        format!(
                            "Can not load page #{} from persistence storage. Attempt #{}. Creating empty page",
                            page_id, attempt_no
                        ),
                        Some(format!("{:?}", zip_error)),
                    );
                }

                return None;
            }
            _ => {
                if let Some(logs) = logs {
                    logs.add_error(
                        Some(topic.topic_id.to_string()),
                        crate::app::logs::SystemProcess::Init,
                        "get_page".to_string(),
                        format!(
                            "Can not load page #{} from persistence storage. Attempt #{}. Retrying...",
                            page_id, attempt_no
                        ),
                        Some(format!("{:?}", err)),
                    );
                }
            }
        }

        attempt_no += 1;

        if attempt_no == 5 {
            return None;
        }
        tokio::time::sleep(Duration::from_secs(1)).await
    }
}
