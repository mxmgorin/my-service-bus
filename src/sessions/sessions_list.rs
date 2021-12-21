use std::{collections::HashMap, sync::Arc, time::Duration};

use rust_extensions::date_time::DateTimeAsMicroseconds;
use tokio::sync::RwLock;

use super::MyServiceBusSession;

struct SessionsListData {
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

    pub async fn get(&self, session_id: i64) -> Option<Arc<MyServiceBusSession>> {
        let read_access = self.data.read().await;
        return Some(read_access.sessions.get(&session_id)?.clone());
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

    pub async fn get_snapshot(&self) -> (usize, Vec<Arc<MyServiceBusSession>>) {
        let read_access = self.data.read().await;

        let mut sessions_result = Vec::new();

        for session in read_access.sessions.values() {
            sessions_result.push(session.clone());
        }

        return (read_access.snapshot_id, sessions_result);
    }

    pub async fn get_dead_connections(
        &self,
        timeout: Duration,
    ) -> Option<Vec<Arc<MyServiceBusSession>>> {
        let now = DateTimeAsMicroseconds::now();

        let mut result = None;

        let read_access = self.data.read().await;

        for session in read_access.sessions.values() {
            let last_incoming_package = session.last_incoming_package.as_date_time();

            if now.duration_since(last_incoming_package) > timeout {
                if result.is_none() {
                    result = Some(Vec::new());
                }

                result.as_mut().unwrap().push(session.clone());
            }
        }

        result
    }
}
