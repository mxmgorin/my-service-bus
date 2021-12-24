use std::sync::{Arc, Mutex};

use my_service_bus_shared::page_id::PageId;
use my_service_bus_tcp_shared::TcpContract;

use crate::{tcp::tcp_server::ConnectionId, topics::Topic};

use super::DeliveryDependecies;

#[cfg(test)]
pub struct DeliveryDependeciesMock {
    sent_packets: Mutex<Option<Vec<(ConnectionId, TcpContract)>>>,
    load_page_event_data: Mutex<Option<(Arc<Topic>, PageId)>>,
    max_packet_size: usize,
}

#[cfg(test)]
impl DeliveryDependeciesMock {
    pub fn new(max_packet_size: usize) -> Self {
        Self {
            sent_packets: Mutex::new(Some(Vec::new())),
            load_page_event_data: Mutex::new(None),
            max_packet_size,
        }
    }

    pub fn get_sent_packets(&self) -> Vec<(ConnectionId, TcpContract)> {
        let mut sent_packets = self.sent_packets.lock().unwrap();

        let mut result = None;

        std::mem::swap(&mut result, &mut sent_packets);

        result.unwrap()
    }

    pub fn get_load_page_event_data(&self) -> (Arc<Topic>, PageId) {
        let mut write_access = self.load_page_event_data.lock().unwrap();

        let mut result = None;

        std::mem::swap(&mut result, &mut write_access);

        result.unwrap()
    }
}

#[cfg(test)]
impl DeliveryDependecies for DeliveryDependeciesMock {
    fn get_max_delivery_size(&self) -> usize {
        self.max_packet_size
    }

    fn send_package(&self, session_id: ConnectionId, tcp_packet: TcpContract) {
        let mut sent_packets = self.sent_packets.lock().unwrap();
        sent_packets
            .as_mut()
            .unwrap()
            .push((session_id, tcp_packet));
    }

    fn load_page(&self, topic: Arc<Topic>, page_id: PageId) {
        let mut write_access = self.load_page_event_data.lock().unwrap();

        if write_access.is_some() {
            panic!("We have already data");
        }

        *write_access = Some((topic, page_id));
    }
}
