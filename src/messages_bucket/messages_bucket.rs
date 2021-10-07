use std::{collections::BTreeMap, sync::Arc};

use my_service_bus_shared::{page_id::PageId, MessageId};

use crate::message_pages::MessagesPage;

use super::MessagesBucketPage;

pub struct MessagesBucket {
    pub pages: BTreeMap<PageId, MessagesBucketPage>,
    pub min_id: Option<MessageId>,
    pub total_size: usize,
}

impl MessagesBucket {
    pub fn new() -> Self {
        Self {
            pages: BTreeMap::new(),
            min_id: None,
            total_size: 0,
        }
    }

    pub fn has_page(&mut self, page_id: PageId) -> bool {
        self.pages.contains_key(&page_id)
    }

    pub fn add_page(&mut self, page: Arc<MessagesPage>) {
        let page = MessagesBucketPage::new(page);
        self.pages.insert(page.page.page_id, page);
    }

    pub fn get_page(&mut self, page_id: PageId) -> &mut MessagesBucketPage {
        return self.pages.get_mut(&page_id).unwrap();
    }

    pub fn messages_count(&self) -> usize {
        let mut result = 0;

        for page in self.pages.values() {
            result += page.messages_count();
        }

        result
    }

    #[inline]
    fn update_min_id(&mut self, msg_id: MessageId) {
        if let Some(min_id) = self.min_id {
            if min_id > msg_id {
                self.min_id = Some(msg_id);
            }
        } else {
            self.min_id = Some(msg_id)
        }
    }

    pub fn add_total_size(&mut self, msg_id: MessageId, msg_size: usize) {
        self.total_size += msg_size;

        self.update_min_id(msg_id);
    }

    pub fn remove_message(&mut self, page_id: PageId, msg_id: MessageId) -> bool {
        let page = self.pages.get_mut(&page_id);

        if page.is_none() {
            return false;
        }

        let page = page.unwrap();

        if page.page.page_id != page_id {
            println!("Somehow we are here");
            return false;
        }

        let removed = page.remove(msg_id);

        if removed.is_none() {
            return false;
        }

        let removed = removed.unwrap();

        self.total_size -= removed.msg_size;

        return true;
    }
}
