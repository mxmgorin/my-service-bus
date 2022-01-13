use std::{collections::HashMap, sync::Arc};

use my_tcp_sockets::ConnectionId;

use super::{MyServiceBusSession, SessionId};

pub struct SessionsListData {
    snapshot_id: usize,
    sessions: HashMap<SessionId, Arc<MyServiceBusSession>>,
    tcp_sessions: HashMap<ConnectionId, Arc<MyServiceBusSession>>,
    http_sessions: HashMap<String, Arc<MyServiceBusSession>>,
    #[cfg(test)]
    test_sessions: HashMap<u8, Arc<MyServiceBusSession>>,
    current_session_id: SessionId,
}

impl SessionsListData {
    pub fn new() -> Self {
        Self {
            snapshot_id: 0,
            sessions: HashMap::new(),
            current_session_id: 0,
            tcp_sessions: HashMap::new(),
            #[cfg(test)]
            test_sessions: HashMap::new(),
            http_sessions: HashMap::new(),
        }
    }
    pub fn get_next_session_id(&mut self) -> SessionId {
        let result = self.current_session_id;
        self.current_session_id += 1;
        result
    }

    pub fn add(&mut self, session: Arc<MyServiceBusSession>) {
        self.sessions.insert(session.id, session.clone());
        self.snapshot_id += 1;

        match &session.connection {
            super::SessionConnection::Tcp(data) => {
                self.tcp_sessions.insert(data.connection.id, session);
            }
            super::SessionConnection::Http(data) => {
                self.http_sessions.insert(data.id.to_string(), session);
            }
            #[cfg(test)]
            super::SessionConnection::Test(connection) => {
                self.test_sessions.insert(connection.id, session);
            }
        }
    }

    pub fn get(&self, session_id: SessionId) -> Option<Arc<MyServiceBusSession>> {
        let result = self.sessions.get(&session_id)?;
        Some(result.clone())
    }

    pub fn get_by_tcp_connection_id(
        &self,
        connection_id: ConnectionId,
    ) -> Option<Arc<MyServiceBusSession>> {
        let result = self.tcp_sessions.get(&connection_id)?;
        Some(result.clone())
    }

    pub fn get_by_http_session(&self, session_id: &str) -> Option<Arc<MyServiceBusSession>> {
        let result = self.http_sessions.get(session_id)?;
        Some(result.clone())
    }

    pub fn get_session_id_by_tcp_connection(
        &self,
        connection_id: ConnectionId,
    ) -> Option<SessionId> {
        let result = self.tcp_sessions.get(&connection_id)?;
        Some(result.id)
    }

    fn remove(&mut self, session_id: SessionId) -> Option<Arc<MyServiceBusSession>> {
        let removed_session = self.sessions.remove(&session_id);

        if let Some(session) = removed_session {
            self.snapshot_id += 1;
            match &session.connection {
                super::SessionConnection::Tcp(data) => {
                    self.tcp_sessions.remove(&data.connection.id);
                }
                super::SessionConnection::Http(data) => {
                    self.http_sessions.remove(&data.id);
                }
                #[cfg(test)]
                super::SessionConnection::Test(connection) => {
                    self.test_sessions.remove(&connection.id);
                }
            }
            Some(session)
        } else {
            None
        }
    }

    pub fn remove_tcp(&mut self, connection_id: ConnectionId) -> Option<Arc<MyServiceBusSession>> {
        let session_id = self.tcp_sessions.get(&connection_id)?.id;
        return self.remove(session_id);
    }

    pub fn get_snapshot(&self) -> (usize, Vec<Arc<MyServiceBusSession>>) {
        let mut sessions_result = Vec::new();

        for session in self.sessions.values() {
            sessions_result.push(session.clone());
        }

        return (self.snapshot_id, sessions_result);
    }

    pub fn one_second_tick(&self) {
        for session in self.http_sessions.values() {
            if let super::SessionConnection::Http(data) = &session.connection {
                data.one_second_tick();
            }
        }
    }
}
