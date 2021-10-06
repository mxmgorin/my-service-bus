use std::collections::HashMap;

use crate::app::AppContext;
use crate::topics::Topic;

pub async fn execute(app: &AppContext, topic: &Topic) {
    let active_pages = topic.get_active_pages().await;
    load_pages(app, topic, &active_pages).await;
    gc_pages(app, topic, &active_pages).await;
}

async fn load_pages(app: &AppContext, topic: &Topic, active_pages: &HashMap<i64, i64>) {
    for page_id in active_pages.keys() {
        if !topic.messages.has_page(page_id).await {
            print!(
                "Loading page {}/{} as warm up process",
                topic.topic_id, page_id
            );
            crate::operations::message_pages::restore_page(app, topic, *page_id, "load_pages")
                .await;
        }
    }
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
