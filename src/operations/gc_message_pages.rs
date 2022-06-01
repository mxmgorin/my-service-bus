use crate::app::AppContext;
use crate::topics::TopicData;

pub fn gc_message_pages(app: &AppContext, topic_data: &mut TopicData) {
    let active_pages = topic_data.get_active_pages();

    let pages = topic_data.pages.get_pages_info();

    for page in pages {
        if !active_pages.contains_key(&page.page_no) {
            if page.persist_size == 0 {
                topic_data.pages.remove_page(&page.page_no);
            } else {
                app.logs
                    .add_info(
                        Some(topic_data.topic_id.to_string()),
                        crate::app::logs::SystemProcess::Timer,
                        format!("Gc Page: {}", page.page_no),
                        format!(
                            "Page can not gced since it has {} messages to persist. Skipping this iteration...",
                            page.persist_size
                        ),
                    );
            }
        }
    }
}
