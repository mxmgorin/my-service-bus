use std::{collections::HashMap, sync::Arc};

use my_service_bus_tcp_shared::TcpContract;
use tokio::{io::WriteHalf, net::TcpStream, sync::RwLock};

use crate::{
    app::logs::Logs,
    date_time::{AtomicDateTime, MyDateTime},
    operations::OperationFailResult,
    subscribers::SubscriberId,
};

use super::{MySbSessionStatistic, MySbSessionSubscriberData, MyServiceBusSessionData};

pub type ConnectionId = i64;

pub struct MyServiceBusSession {
    pub data: RwLock<MyServiceBusSessionData>,
    pub ip: String,
    pub id: ConnectionId,
    pub connected: MyDateTime,
    pub last_incoming_package: AtomicDateTime,

    pub logs: Arc<Logs>,
}

const BADGE_HIGHLIGHT_TIMOUT: u8 = 2;

impl MyServiceBusSession {
    pub fn new(
        id: ConnectionId,
        ip: String,
        tcp_stream: WriteHalf<TcpStream>,
        logs: Arc<Logs>,
    ) -> Self {
        let now = MyDateTime::utc_now();

        let data = MyServiceBusSessionData::new(tcp_stream);

        Self {
            id,
            ip,
            data: RwLock::new(data),
            connected: now,
            last_incoming_package: AtomicDateTime::from_date_time(now),
            logs,
        }
    }

    pub async fn increase_read_size(&self, read_size: usize) {
        let mut data = self.data.write().await;
        data.statistic.increase_read_size(read_size).await;
    }

    pub async fn set_socket_name(&self, set_socket_name: String, client_version: Option<String>) {
        let mut data = self.data.write().await;
        data.name = Some(set_socket_name);
        data.client_version = client_version;
    }

    pub async fn set_protocol_version(&self, protocol_version: i32) {
        let mut data = self.data.write().await;
        data.attr.protocol_version = protocol_version;
    }

    pub async fn update_packet_versions(&self, packet_versions: &HashMap<u8, i32>) {
        let mut data = self.data.write().await;
        data.attr.versions.update(packet_versions);
    }

    pub async fn get_statistic(&self) -> MySbSessionStatistic {
        let read_access = self.data.read().await;
        read_access.statistic.clone()
    }

    pub async fn one_second_tick(&self) {
        let mut write_access = self.data.write().await;
        write_access.statistic.one_second_tick();
    }

    pub async fn get_name(&self) -> String {
        let data = self.data.read().await;

        match &data.name {
            Some(name) => return format!("{} {}", name, self.ip),
            None => self.ip.clone(),
        }
    }

    async fn serialize_tcp_contract(&self, tcp_contract: TcpContract) -> Vec<u8> {
        let data = self.data.read().await;
        tcp_contract.serialize(&data.attr)
    }

    pub async fn send(&self, tcp_contract: TcpContract) {
        let buf = self.serialize_tcp_contract(tcp_contract).await;

        let mut write_access = self.data.write().await;
        write_access.send(buf.as_ref(), self.logs.as_ref()).await;
    }

    pub async fn add_publisher(&self, topic: &str) {
        let mut data = self.data.write().await;

        if !data.statistic.publishers.contains_key(topic) {
            data.statistic
                .publishers
                .insert(topic.to_string(), BADGE_HIGHLIGHT_TIMOUT);
        }
    }

    pub async fn topic_has_activity(&self, topic: &str) {
        let mut data = self.data.write().await;
        data.statistic
            .publishers
            .insert(topic.to_string(), BADGE_HIGHLIGHT_TIMOUT);
    }

    pub async fn add_subscriber(
        &self,
        subscriber_id: SubscriberId,
        topic_id: &str,
        queue_id: &str,
    ) -> Result<(), OperationFailResult> {
        let mut statistic_write_access = self.data.write().await;
        if statistic_write_access.is_disconnected() {
            return Err(OperationFailResult::SessionIsDisconnected);
        }
        statistic_write_access.statistic.subscribers.insert(
            subscriber_id,
            MySbSessionSubscriberData::new(topic_id, queue_id, 0),
        );

        let mut data = self.data.write().await;
        data.add_subscriber(&subscriber_id, topic_id, queue_id);

        return Ok(());
    }

    pub async fn set_delivered_statistic(
        &self,
        subscriber_id: i64,
        delivered: usize,
        microseconds: usize,
    ) {
        let mut write_access = self.data.write().await;

        let found_subscriber = write_access.statistic.subscribers.get_mut(&subscriber_id);

        if let Some(subscriber) = found_subscriber {
            subscriber.delivered_amount.increase(delivered);
            subscriber.delivery_microseconds.increase(microseconds);
        }
    }

    pub async fn set_not_delivered_statistic(
        &self,
        subscriber_id: i64,
        delivered: i32,
        microseconds: i32,
    ) {
        let mut write_access = self.data.write().await;

        let found_subscriber = write_access.statistic.subscribers.get_mut(&subscriber_id);

        if let Some(subscriber) = found_subscriber {
            subscriber.metrics.put(microseconds / -delivered)
        }
    }

    pub async fn remove_subscriber(&self, subscriber_id: SubscriberId) {
        let mut statistic_write_access = self.data.write().await;
        statistic_write_access
            .statistic
            .subscribers
            .remove(&subscriber_id);

        statistic_write_access.remove_subscriber(&subscriber_id);
    }

    pub async fn disconnect(&self) -> Option<HashMap<SubscriberId, MySbSessionSubscriberData>> {
        let mut write_access = self.data.write().await;

        write_access.disconnect(self.logs.as_ref()).await;

        return Some(write_access.get_subscribers());
    }

    pub async fn set_on_delivery_flag(&self, subscriber_id: SubscriberId) {
        let mut statistic_write_access = self.data.write().await;

        let subscriber = statistic_write_access
            .statistic
            .subscribers
            .get_mut(&subscriber_id);

        if let Some(subscriber) = subscriber {
            subscriber.active = 2;
        }
    }
}
