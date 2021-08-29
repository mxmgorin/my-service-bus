use my_service_bus_shared::page_id::PageId;
use tokio::sync::RwLock;

use crate::messages::MySbMessageContent;
use std::{collections::HashMap, sync::Arc};

use super::MessagesPage;

pub struct PageInfo {
    pub page_no: PageId,
    pub page_size: usize,
    pub count: usize,
    pub persist_size: i64,
    pub is_being_persisted: bool,
}

pub struct MessagesPagesCache {
    pub pages: RwLock<HashMap<PageId, Arc<MessagesPage>>>,
}

impl MessagesPagesCache {
    pub fn new() -> Self {
        Self {
            pages: RwLock::new(HashMap::new()),
        }
    }

    pub async fn new_messages(&self, msgs_by_pages: HashMap<PageId, Vec<MySbMessageContent>>) {
        let mut write_access = self.pages.write().await;

        for (page_id, msgs) in msgs_by_pages {
            if !write_access.contains_key(&page_id) {
                write_access.insert(page_id, Arc::new(MessagesPage::new(page_id)));
            }

            write_access.get(&page_id).unwrap().new_messages(msgs).await;
        }
    }

    pub async fn get(&self, page_id: PageId) -> Option<Arc<MessagesPage>> {
        let read_access = self.pages.read().await;
        let result = read_access.get(&page_id)?;
        Some(result.clone())
    }

    pub async fn has_page(&self, page_id: &PageId) -> bool {
        let read_access = self.pages.read().await;
        read_access.contains_key(&page_id)
    }

    pub async fn get_pages_info(&self) -> Vec<PageInfo> {
        let read_access = self.pages.read().await;

        let mut result = Vec::new();

        for (page_id, page) in read_access.iter() {
            let read_access = page.data.read().await;
            result.push(PageInfo {
                page_no: page_id.clone(),
                page_size: read_access.size,
                count: read_access.messages.len(),
                persist_size: read_access.to_be_persisted.len(),
                is_being_persisted: read_access.is_being_persisted,
            });
        }

        return result;
    }

    pub async fn get_pages(&self) -> Vec<Arc<MessagesPage>> {
        let read_access = self.pages.read().await;

        let mut result = Vec::new();

        for page in read_access.values() {
            result.push(page.clone());
        }

        result
    }

    pub async fn remove_page(&self, page_id: &PageId) {
        let mut write_access = self.pages.write().await;

        write_access.remove(page_id);
    }

    pub async fn restore_page(&self, page: MessagesPage) {
        let mut write_access = self.pages.write().await;

        write_access.insert(page.page_id, Arc::new(page));
    }

    pub async fn get_persist_queue_size(&self) -> i64 {
        let mut result = 0;
        let read_access = self.pages.read().await;

        for page in read_access.values() {
            let read_access = page.data.read().await;
            result += read_access.to_be_persisted.len();
        }

        result
    }
}
