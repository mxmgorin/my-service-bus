use std::sync::Arc;

use my_service_bus_shared::page_id::PageId;
use my_service_bus_tcp_shared::TcpContract;

use crate::{tcp::tcp_server::ConnectionId, topics::Topic};

pub trait DeliveryDependecies {
    fn get_max_delivery_size(&self) -> usize;
    fn send_package(&self, session_id: ConnectionId, tcp_packet: TcpContract);
    fn load_page(&self, topic: Arc<Topic>, page_id: PageId);
}
