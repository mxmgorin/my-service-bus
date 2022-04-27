use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use tokio::sync::{Mutex, MutexGuard};

use super::TopicData;

pub struct TopicDataAccess<'s> {
    topic_data: MutexGuard<'s, TopicData>,
    mutex: Arc<Mutex<Vec<String>>>,
    process: String,
}

impl<'s> TopicDataAccess<'s> {
    pub fn new(
        topic_data: MutexGuard<'s, TopicData>,
        mutex: Arc<Mutex<Vec<String>>>,
        process: String,
    ) -> Self {
        Self {
            topic_data,
            mutex,
            process,
        }
    }
}

impl<'s> Deref for TopicDataAccess<'s> {
    type Target = MutexGuard<'s, TopicData>;

    fn deref(&self) -> &Self::Target {
        &self.topic_data
    }
}

impl<'s> DerefMut for TopicDataAccess<'s> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.topic_data
    }
}

impl<'s> Drop for TopicDataAccess<'s> {
    fn drop(&mut self) {
        tokio::spawn(remove_el(self.mutex.clone(), self.process.clone()));
    }
}

async fn remove_el(mutex: Arc<Mutex<Vec<String>>>, process: String) {
    let mut write_access = mutex.lock().await;

    if let Some(index) = write_access.iter().position(|itm| itm == process.as_str()) {
        write_access.remove(index);
    }
}
