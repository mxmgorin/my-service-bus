use std::collections::HashMap;

use my_service_bus_shared::sub_page::SubPageId;
use rust_extensions::lazy::LazyVec;

use crate::app::AppContext;
use crate::topics::TopicData;

pub fn gc_message_pages(app: &AppContext, topic_data: &mut TopicData) {
    let active_pages = super::get_active_sub_pages(topic_data);

    let sub_pages_to_gc = get_subpages_to_gc(topic_data, &active_pages);

    if sub_pages_to_gc.is_none() {
        return;
    }

    for sub_page_to_gc in sub_pages_to_gc.unwrap() {
        todo!("Implement")
    }
}

fn get_subpages_to_gc(
    topic_data: &TopicData,
    active_pages: &HashMap<usize, SubPageId>,
) -> Option<Vec<SubPageId>> {
    let mut result = LazyVec::new();

    for page in topic_data.pages.get_pages() {
        for sub_page in page.sub_pages.values() {
            if !active_pages.contains_key(&sub_page.sub_page.sub_page_id.value) {
                if sub_page.messages_to_persist.len() == 0 {
                    result.add(sub_page.sub_page.sub_page_id);
                }
            }
        }
    }

    result.get_result()
}
