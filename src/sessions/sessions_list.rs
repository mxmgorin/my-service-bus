use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use super::MyServiceBusSession;

pub type SessionId = i32;

struct SessionsListData {
    snapshot_id: usize,
    sessions: HashMap<SessionId, Arc<MyServiceBusSession>>,
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

    pub async fn get(&self, session_id: i32) -> Option<Arc<MyServiceBusSession>> {
        let read_access = self.data.read().await;
        return Some(read_access.sessions.get(&session_id)?.clone());
    }

    pub async fn remove(&self, id: &i32) -> Option<Arc<MyServiceBusSession>> {
        let mut write_access = self.data.write().await;
        let result = write_access.sessions.remove(id)?;
        write_access.snapshot_id += 1;

        return Some(result);
    }

    pub async fn get_snapshot(&self) -> (usize, Vec<Arc<MyServiceBusSession>>) {
        let read_access = self.data.read().await;

        let mut sessions_result = Vec::new();

        for session in read_access.sessions.values() {
            sessions_result.push(session.clone());
        }

        return (read_access.snapshot_id, sessions_result);
    }
}
