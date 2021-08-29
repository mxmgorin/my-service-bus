use std::{collections::HashMap, sync::Arc};

use my_service_bus_shared::MessageId;
use tokio::sync::RwLock;

use super::topic::Topic;
use crate::topics::topic_snapshot::TopicSnapshot;

pub struct TopicListData {
    pub topics: HashMap<String, Arc<Topic>>,
    pub snapshot_id: usize,
}

impl TopicListData {
    pub fn new() -> Self {
        Self {
            topics: HashMap::new(),
            snapshot_id: 0,
        }
    }
}

pub struct TopicsList {
    data: RwLock<TopicListData>,
}

impl TopicsList {
    pub fn new() -> Self {
        TopicsList {
            data: RwLock::new(TopicListData::new()),
        }
    }

    pub async fn get(&self, topic_id: &str) -> Option<Arc<Topic>> {
        let read_access = self.data.read().await;

        match read_access.topics.get(topic_id) {
            Some(result) => Some(Arc::clone(result)),
            None => None,
        }
    }

    pub async fn get_all(&self) -> Vec<Arc<Topic>> {
        let mut result = Vec::new();
        let read_access = self.data.read().await;

        for topic in read_access.topics.values() {
            result.push(Arc::clone(topic))
        }

        result
    }

    pub async fn add_if_not_exists(&self, topic_id: &str) -> Arc<Topic> {
        let mut write_access = self.data.write().await;

        if !write_access.topics.contains_key(topic_id) {
            write_access
                .topics
                .insert(topic_id.to_string(), Arc::new(Topic::new(topic_id, 0)));
            write_access.snapshot_id += 1;
        }

        //Safety: This unwrap is handeled - since we create topic by all means during previous if statement.
        let result = write_access.topics.get(topic_id).unwrap();

        return result.clone();
    }

    pub async fn restore(&self, topic_id: &str, message_id: MessageId) -> Arc<Topic> {
        let mut write_access = self.data.write().await;

        let result = Arc::new(Topic::new(topic_id, message_id));

        write_access
            .topics
            .insert(topic_id.to_string(), result.clone());

        write_access.snapshot_id += 1;

        return result;
    }

    pub async fn get_snapshot(&self) -> Vec<TopicSnapshot> {
        let mut result = Vec::new();

        let topics = self.get_all();

        for topic in topics.await {
            let snapshot = topic.get_snapshot().await;
            result.push(snapshot);
        }

        result
    }

    pub async fn get_snapshot_id(&self) -> usize {
        let read_access = self.data.read().await;
        return read_access.snapshot_id;
    }

    pub async fn one_second_tick(&self) {
        let read_access = self.data.read().await;

        for topic in read_access.topics.values() {
            let persist_queue_size = topic.messages.get_persist_queue_size().await;

            topic.metrics.one_second_tick(persist_queue_size).await;
        }
    }
}
