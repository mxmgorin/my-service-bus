use my_service_bus_shared::{page_id::PageId, MessageId};

use super::MessagesBucketPage;

pub struct MessagesBucket {
    pub pages: Vec<MessagesBucketPage>,
    found_page_index: usize,
    pub min_id: Option<MessageId>,
    pub total_size: usize,
}

impl MessagesBucket {
    pub fn new() -> Self {
        Self {
            pages: Vec::new(),
            min_id: None,
            total_size: 0,
            found_page_index: 0,
        }
    }

    pub fn get_last_page_with_id(&mut self, page_id: PageId) -> Option<&mut MessagesBucketPage> {
        let last = self.pages.last_mut()?;

        if last.page.page_id == page_id {
            return Some(last);
        }

        return None;
    }

    pub fn add_page(&mut self, page: MessagesBucketPage) {
        self.pages.push(page);
    }

    pub fn messages_count(&self) -> usize {
        let mut result = 0;

        for page in &self.pages {
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

    pub fn find_page(&mut self, page_id: PageId) -> bool {
        let mut index: usize = 0;

        for page in &mut self.pages {
            if page.page.page_id == page_id {
                self.found_page_index = index;
                return true;
            }

            index += 1;
        }

        return false;
    }

    pub fn remove_message(&mut self, page_id: PageId, msg_id: MessageId) -> bool {
        let page = self.pages.get_mut(self.found_page_index);

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
