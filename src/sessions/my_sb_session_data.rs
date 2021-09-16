use std::collections::HashMap;

use my_service_bus_tcp_shared::ConnectionAttributes;

use crate::subscribers::SubscriberId;

use super::my_sb_session_subscriber_data::MySbSessionSubscriberData;

pub struct MyServiceBusSessionData {
    pub name: Option<String>,
    pub client_version: Option<String>,

    subscribers: HashMap<SubscriberId, MySbSessionSubscriberData>,

    pub attr: ConnectionAttributes,

    pub disconnected: bool,
}

impl MyServiceBusSessionData {
    pub fn new() -> Self {
        Self {
            name: None,
            client_version: None,
            subscribers: HashMap::new(),
            attr: ConnectionAttributes::new(),
            disconnected: false,
        }
    }

    pub fn get_name(&self) -> Option<String> {
        let result = self.name.as_ref()?;
        return Some(result.to_string());
    }

    pub fn get_version(&self) -> Option<String> {
        let result = self.client_version.as_ref()?;
        return Some(result.to_string());
    }

    pub fn has_subscriber(&self, subscriber_id: &SubscriberId) -> bool {
        self.subscribers.contains_key(subscriber_id)
    }

    pub fn add_subscriber(&mut self, subscriber_id: &SubscriberId, topic_id: &str, queue_id: &str) {
        self.subscribers.insert(
            *subscriber_id,
            MySbSessionSubscriberData::new(topic_id, queue_id, 0),
        );
    }

    pub fn remove_subscriber(&mut self, subscriber_id: &SubscriberId) {
        self.subscribers.remove(subscriber_id);
    }

    pub fn get_subscribers(&self) -> HashMap<SubscriberId, MySbSessionSubscriberData> {
        return self.subscribers.clone();
    }
}
