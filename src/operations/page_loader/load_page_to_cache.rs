use std::sync::Arc;

use my_service_bus_shared::{page_id::PageId, sub_page::SubPageId};
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{app::logs::Logs, persistence::MessagesPagesRepo, topics::Topic};

pub async fn load_page_to_cache(
    topic: Arc<Topic>,
    messages_pages_repo: Arc<MessagesPagesRepo>,
    logs: Option<&Logs>,
    page_id: PageId,
    sub_page_id: SubPageId,
) {
    let mut dt = topic.restore_page_lock.lock().await;

    let (min_message_id, topic_message_id) = {
        let topic_data = topic.get_access().await;
        (topic_data.get_min_message_id(), topic_data.message_id)
    };

    let (from_message_id, to_message_id) =
        super::utils::get_load_page_interval(min_message_id, topic_message_id, page_id);

    println!(
        "Loading messages {}-{} for page {} for topic:{}",
        from_message_id, to_message_id, page_id, topic.topic_id
    );

    let sub_page = super::operations::load_page(
        topic.as_ref(),
        &messages_pages_repo,
        logs,
        page_id,
        sub_page_id,
    )
    .await;

    {
        let mut topic_data = topic.get_access().await;
        topic_data.pages.restore_subpage(sub_page);
    }

    *dt = DateTimeAsMicroseconds::now();
}
