use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use super::MyServiceBusSession;

pub struct SessionsListData {
    snapshot_id: usize,
    sessions: HashMap<i64, Arc<MyServiceBusSession>>,
}

pub struct SessionsList {
    data: RwLock<SessionsListData>,
}

impl SessionsList {
    pub fn new() -> Self {
        let data = SessionsListData {
            snapshot_id: 0,
            sessions: HashMap::new(),
        };

        Self {
            data: RwLock::new(data),
        }
    }

    pub async fn add(&self, session: Arc<MyServiceBusSession>) {
        let mut write_access = self.data.write().await;
        write_access.sessions.insert(session.id, session);
        write_access.snapshot_id += 1;
    }

    pub async fn get_snapshot(&self) -> (usize, Vec<Arc<MyServiceBusSession>>) {
        let read_access = self.data.read().await;

        let result = read_access
            .sessions
            .values()
            .into_iter()
            .map(|itm| itm.clone())
            .collect();

        (read_access.snapshot_id, result)
    }

    pub async fn remove(&self, id: &i64) -> Option<Arc<MyServiceBusSession>> {
        let mut write_access = self.data.write().await;

        let result = write_access.sessions.remove(id)?;

        write_access.snapshot_id += 1;

        return Some(result);
    }

    pub async fn one_second_tick(&self) {
        let read_access = self.data.read().await;

        for session in read_access.sessions.values() {
            session.one_second_tick().await;
        }
    }
}
