use std::time::Duration;

use my_service_bus_shared::page_id::PageId;

use crate::{app::AppContext, topics::Topic};

pub async fn do_it(app: &AppContext, topic: &Topic, page_id: PageId) {
    let mut attempt_no = 0;
    loop {
        let result = app
            .messages_pages_repo
            .load_page(topic.topic_id.as_str(), page_id)
            .await;

        if let Ok(page) = result {
            topic.messages.restore_page(page).await;
            return;
        }

        //TODO - Handle Situation - if we do not have page at all - we load empty page

        let err = result.err().unwrap();

        app.logs
            .add_error(
                Some(topic.topic_id.to_string()),
                crate::app::logs::SystemProcess::Init,
                "get_page".to_string(),
                format!(
                    "Can not load page #{} from persistence storage. Attempt #{}",
                    page_id, attempt_no
                ),
                Some(format!("{:?}", err)),
            )
            .await;

        attempt_no += 1;
        tokio::time::sleep(Duration::from_secs(1)).await
    }
}
