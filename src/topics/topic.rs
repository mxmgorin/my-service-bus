use futures_util::lock::Mutex;
use my_service_bus_shared::page_id::{get_page_id, PageId};
use my_service_bus_shared::MessageId;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use super::topic_data::TopicData;

pub struct Topic {
    pub topic_id: String,
    pub data: Mutex<TopicData>,
    pub restore_page_lock: Mutex<DateTimeAsMicroseconds>,
}

impl Topic {
    pub fn new(topic_id: String, message_id: MessageId) -> Self {
        Self {
            topic_id: topic_id.to_string(),
            data: Mutex::new(TopicData::new(topic_id, message_id)),
            restore_page_lock: Mutex::new(DateTimeAsMicroseconds::now()),
        }
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
}
