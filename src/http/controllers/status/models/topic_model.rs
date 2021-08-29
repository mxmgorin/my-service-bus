use std::sync::Arc;

use crate::{app::AppContext, topics::Topic};

use my_service_bus_shared::{page_id::PageId, MessageId};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct TopicsJsonResult {
    pub items: Vec<TopicJsonContract>,
    #[serde(rename = "snapshotId")]
    pub snapshot_id: usize,
}

impl TopicsJsonResult {
    pub async fn new(app: &AppContext, topics: &[Arc<Topic>]) -> Self {
        let mut items = Vec::new();

        for topic in topics {
            let metrics = topic.metrics.get().await;

            let pages = topic.messages.get_pages_info().await;

            items.push(TopicJsonContract {
                id: topic.topic_id.to_string(),
                message_id: topic.get_message_id().await,
                packets_per_second: metrics.packets_per_second,
                messages_per_second: metrics.messages_per_second,
                publish_history: metrics.publish_history,
                persist_size: metrics.persist_queue_size,
                pages: pages
                    .iter()
                    .map(|itm| TopicPageJsonContract {
                        id: itm.page_no,
                        percent: itm.count / 1000,
                        size: itm.page_size,
                    })
                    .collect(),
            })
        }

        Self {
            snapshot_id: app.topic_list.get_snapshot_id().await,
            items,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TopicJsonContract {
    pub id: String,
    #[serde(rename = "messageId")]
    pub message_id: MessageId,
    #[serde(rename = "packetPerSec")]
    pub packets_per_second: usize,
    #[serde(rename = "messagesPerSec")]
    pub messages_per_second: usize,
    pub pages: Vec<TopicPageJsonContract>,
    #[serde(rename = "persistSize")]
    pub persist_size: i64,
    #[serde(rename = "publishHistory")]
    pub publish_history: Vec<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TopicPageJsonContract {
    pub id: PageId,
    pub percent: usize,
    pub size: usize,
}
