use std::sync::Arc;

use my_service_bus_shared::page_id::{get_page_id, PageId};
use my_service_bus_shared::MessageId;
use rust_extensions::date_time::DateTimeAsMicroseconds;
use tokio::sync::Mutex;

use super::topic_data::TopicData;
use super::topic_data_access::TopicDataAccess;
use super::TopicSnapshot;

pub struct Topic {
    pub topic_id: String,
    data: Mutex<TopicData>,
    pub restore_page_lock: Mutex<DateTimeAsMicroseconds>,

    topic_data_access: Arc<Mutex<Vec<String>>>,
}

impl Topic {
    pub fn new(topic_id: String, message_id: MessageId) -> Self {
        Self {
            topic_id: topic_id.to_string(),
            data: Mutex::new(TopicData::new(topic_id, message_id)),
            restore_page_lock: Mutex::new(DateTimeAsMicroseconds::now()),
            topic_data_access: Arc::new(Mutex::new(Vec::new())),
        }
    }

    async fn add_to_topic_data_access(&self, process: &str) {
        let mut write_access = self.topic_data_access.lock().await;
        write_access.push(process.to_string());
    }

    pub async fn get_locks(&self) -> Option<Vec<String>> {
        let read_access = self.topic_data_access.lock().await;

        if read_access.len() == 0 {
            return None;
        }

        return Some(read_access.clone());
    }

    pub async fn get_access<'s>(&'s self, process: &str) -> TopicDataAccess<'s> {
        self.add_to_topic_data_access(process).await;
        let access = self.data.lock().await;
        TopicDataAccess::new(access, self.topic_data_access.clone(), process.to_string())
    }

    pub async fn get_message_id(&self) -> MessageId {
        let read_access = self.data.lock().await;
        read_access.message_id
    }

    pub async fn get_current_page(&self) -> PageId {
        let read_access = self.data.lock().await;

        get_page_id(read_access.message_id)
    }

    pub async fn one_second_tick(&self) {
        let mut write_access = self.data.lock().await;
        write_access.one_second_tick();
    }

    pub async fn get_topic_snapshot(&self) -> TopicSnapshot {
        let topic_data = self.data.lock().await;

        TopicSnapshot {
            message_id: topic_data.message_id,
            topic_id: topic_data.topic_id.to_string(),
            queues: topic_data.queues.get_snapshot_to_persist(),
        }
    }
}
