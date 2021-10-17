use std::collections::HashMap;
use std::sync::Arc;

use crate::app::AppContext;
use crate::topics::Topic;

pub async fn execute(app: Arc<AppContext>, topic: Arc<Topic>) {
    let active_pages = topic.get_active_pages().await;
    gc_pages(app.as_ref(), topic.as_ref(), &active_pages).await;
}

async fn gc_pages(app: &AppContext, topic: &Topic, active_pages: &HashMap<i64, i64>) {
    let pages = topic.messages.get_pages_info().await;

    for page in pages {
        if !active_pages.contains_key(&page.page_no) {
            if page.persist_size == 0 {
                topic.messages.remove_page(&page.page_no).await;
            } else {
                app.logs
                    .add_info(
                        Some(topic.topic_id.to_string()),
                        crate::app::logs::SystemProcess::Timer,
                        format!("Gc Page: {}", page.page_no),
                        format!(
                            "Page can not gced since it has {} messages to persist. Skipping this iteration...",
                            page.persist_size
                        ),
                    )
                    .await
            }
        }
    }
}
