use std::{collections::HashMap, sync::Arc};

use my_service_bus_tcp_shared::PacketProtVer;
use tokio::sync::RwLock;

use crate::subscribers::SubscriberId;

use super::{my_sb_session::ConnectionId, MyServiceBusSession};

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

    pub async fn get_by_id(&self, id: ConnectionId) -> Option<Arc<MyServiceBusSession>> {
        let read_access = self.data.read().await;

        let session = read_access.sessions.get(&id)?;

        Some(session.clone())
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

    pub async fn get_packet_and_protocol_version(
        &self,
        subscriber_id: SubscriberId,
        packet: u8,
    ) -> PacketProtVer {
        let mut packet_version = 0;
        let mut protocol_version = 0;

        let read_access = self.data.read().await;

        for session in read_access.sessions.values() {
            let lock_id = session
                .app
                .enter_lock(format!(
                    "SessionsList[{}].get_packet_and_protocol_version",
                    subscriber_id
                ))
                .await;
            let read_access = session.data.read().await;
            if read_access.has_subscriber(&subscriber_id) {
                packet_version = read_access.attr.versions.get_packet_version(packet);
                protocol_version = read_access.attr.protocol_version;
            }

            session.app.exit_lock(lock_id).await;
        }

        return PacketProtVer {
            packet_version,
            protocol_version,
        };
    }
}
