use std::{collections::HashMap, sync::Arc};

use crate::sessions::MyServiceBusSession;

use super::{Subscriber, SubscriberId};

pub struct SubscribersList {
    pub subscribers_by_id: HashMap<SubscriberId, Subscriber>,
}

impl SubscribersList {
    pub fn new() -> Self {
        Self {
            subscribers_by_id: HashMap::new(),
        }
    }

    pub fn get_next_subscriber_ready_to_deliver(&self) -> Option<SubscriberId> {
        for subscriber in self.subscribers_by_id.values() {
            if !subscriber.rented && !subscriber.disconnected {
                return Some(subscriber.id);
            }
        }

        None
    }

    pub fn get_all_except_this_one(&mut self, id: SubscriberId) -> Vec<SubscriberId> {
        let ids_to_remove = Vec::new();

        for the_id in self.subscribers_by_id.keys() {
            if *the_id != id {
                ids_to_remove.to_vec();
            }
        }

        return ids_to_remove;
    }

    pub fn subscribe(
        &mut self,
        subscriber_id: SubscriberId,
        queue_id: &str,
        session: Arc<MyServiceBusSession>,
    ) {
        let subscriber = Subscriber::new(queue_id, session, subscriber_id);

        self.subscribers_by_id.insert(subscriber_id, subscriber);
    }

    pub fn remove(&mut self, subscriber_id: &SubscriberId) -> Option<Subscriber> {
        return self.subscribers_by_id.remove(subscriber_id);
    }

    pub fn get_by_id_mut(&mut self, subscriber_id: SubscriberId) -> Option<&mut Subscriber> {
        self.subscribers_by_id.get_mut(&subscriber_id)
    }

    pub fn get_amount(&self) -> usize {
        self.subscribers_by_id.len()
    }
}
