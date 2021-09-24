use std::{collections::HashMap, sync::Arc};

use my_service_bus_tcp_shared::TcpContract;
use tokio::{io::WriteHalf, net::TcpStream, sync::RwLock};

use crate::{
    app::AppContext,
    date_time::{AtomicDateTime, MyDateTime},
    operations::OperationFailResult,
    subscribers::SubscriberId,
};

use super::{MySbSessionSubscriberData, MyServiceBusSessionData};

pub type ConnectionId = i64;

pub struct MyServiceBusSession {
    pub data: RwLock<MyServiceBusSessionData>,
    pub ip: String,
    pub id: ConnectionId,
    pub connected: MyDateTime,
    pub last_incoming_package: AtomicDateTime,

    pub app: Arc<AppContext>,
}

const BADGE_HIGHLIGHT_TIMOUT: u8 = 2;

impl MyServiceBusSession {
    pub fn new(
        id: ConnectionId,
        ip: String,
        tcp_stream: WriteHalf<TcpStream>,
        app: Arc<AppContext>,
    ) -> Self {
        let now = MyDateTime::utc_now();

        let data = MyServiceBusSessionData::new(tcp_stream);

        Self {
            id,
            ip,
            data: RwLock::new(data),
            connected: now,
            last_incoming_package: AtomicDateTime::from_date_time(now),
            app,
        }
    }

    pub async fn increase_read_size(&self, read_size: usize) {
        let lock_id = self.app.enter_lock("MySbSession.increase_read_size").await;
        let mut data = self.data.write().await;
        data.statistic.increase_read_size(read_size).await;

        self.app.exit_lock(lock_id).await;
    }

    pub async fn set_socket_name(&self, set_socket_name: String, client_version: Option<String>) {
        let lock_id = self.app.enter_lock("MySbSession.set_socket_name").await;

        let mut data = self.data.write().await;
        data.name = Some(set_socket_name);
        data.client_version = client_version;

        self.app.exit_lock(lock_id).await;
    }

    pub async fn set_protocol_version(&self, protocol_version: i32) {
        let lock_id = self
            .app
            .enter_lock("MySbSession.set_protocol_version")
            .await;

        let mut data = self.data.write().await;
        data.attr.protocol_version = protocol_version;

        self.app.exit_lock(lock_id).await;
    }

    pub async fn update_packet_versions(&self, packet_versions: &HashMap<u8, i32>) {
        let lock_id = self
            .app
            .enter_lock("MySbSession.update_packet_versions")
            .await;
        let mut data = self.data.write().await;
        data.attr.versions.update(packet_versions);
        self.app.exit_lock(lock_id).await;
    }

    pub async fn one_second_tick(&self) {
        let lock_id = self.app.enter_lock("MySbSession.one_second_tick").await;
        let mut write_access = self.data.write().await;
        write_access.statistic.one_second_tick();
        self.app.exit_lock(lock_id).await;
    }

    pub async fn get_name(&self) -> String {
        let lock_id = self.app.enter_lock("MySbSession.get_name").await;

        let data = self.data.read().await;

        let result = match &data.name {
            Some(name) => format!("{} {}", name, self.ip),
            None => self.ip.clone(),
        };

        self.app.exit_lock(lock_id).await;

        result
    }

    async fn serialize_tcp_contract(&self, tcp_contract: TcpContract) -> Vec<u8> {
        let data = self.data.read().await;
        tcp_contract.serialize(&data.attr)
    }

    pub async fn send(&self, tcp_contract: TcpContract) {
        let buf = self.serialize_tcp_contract(tcp_contract).await;

        let lock_id = self.app.enter_lock("MySbSession.send").await;

        let mut write_access = self.data.write().await;
        write_access
            .send(buf.as_ref(), self.app.logs.as_ref())
            .await;

        self.app.exit_lock(lock_id).await;
    }

    pub async fn send_and_set_on_delivery(
        &self,
        tcp_contract: TcpContract,
        subscriber_id: SubscriberId,
    ) {
        let buf = self.serialize_tcp_contract(tcp_contract).await;

        let lock_id = self
            .app
            .enter_lock("MySbSession.send_and_set_on_delivery")
            .await;

        let mut write_access = self.data.write().await;
        write_access
            .send(buf.as_ref(), self.app.logs.as_ref())
            .await;

        write_access.set_on_delivery_flag(subscriber_id);

        self.app.exit_lock(lock_id).await;
    }

    pub async fn add_publisher(&self, topic: &str) {
        let lock_id = self.app.enter_lock("MySbSession.add_publisher").await;
        let mut data = self.data.write().await;

        data.statistic
            .publishers
            .insert(topic.to_string(), BADGE_HIGHLIGHT_TIMOUT);

        if !data.statistic.publishers.contains_key(topic) {
            data.statistic
                .publishers
                .insert(topic.to_string(), BADGE_HIGHLIGHT_TIMOUT);
        }

        self.app.exit_lock(lock_id).await;
    }

    pub async fn add_subscriber(
        &self,
        subscriber_id: SubscriberId,
        topic_id: &str,
        queue_id: &str,
    ) -> Result<(), OperationFailResult> {
        let lock_id = self.app.enter_lock("MySbSession.add_subscriber").await;

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

        self.app.exit_lock(lock_id).await;
        return Ok(());
    }

    pub async fn set_delivered_statistic(
        &self,
        subscriber_id: i64,
        delivered: usize,
        microseconds: usize,
    ) {
        let lock_id = self
            .app
            .enter_lock("MySbSession.set_delivered_statistic")
            .await;

        let mut write_access = self.data.write().await;

        let found_subscriber = write_access.statistic.subscribers.get_mut(&subscriber_id);

        if let Some(subscriber) = found_subscriber {
            subscriber.delivered_amount.increase(delivered);
            subscriber.delivery_microseconds.increase(microseconds);
        }

        self.app.exit_lock(lock_id).await;
    }

    pub async fn set_not_delivered_statistic(
        &self,
        subscriber_id: i64,
        delivered: i32,
        microseconds: i32,
    ) {
        let lock_id = self
            .app
            .enter_lock("MySbSession.set_not_delivered_statistic")
            .await;

        let mut write_access = self.data.write().await;

        let found_subscriber = write_access.statistic.subscribers.get_mut(&subscriber_id);

        if let Some(subscriber) = found_subscriber {
            subscriber.metrics.put(microseconds / -delivered)
        }

        self.app.exit_lock(lock_id).await;
    }

    pub async fn remove_subscriber(&self, subscriber_id: SubscriberId) {
        let lock_id = self.app.enter_lock("MySbSession.remove_subscriber").await;
        let mut statistic_write_access = self.data.write().await;
        statistic_write_access
            .statistic
            .subscribers
            .remove(&subscriber_id);

        statistic_write_access.remove_subscriber(&subscriber_id);
        self.app.exit_lock(lock_id).await;
    }

    pub async fn disconnect(&self) -> Option<HashMap<SubscriberId, MySbSessionSubscriberData>> {
        let lock_id = self.app.enter_lock("MySbSession.disconnect").await;

        let mut write_access = self.data.write().await;

        write_access.disconnect(self.app.logs.as_ref()).await;

        self.app.exit_lock(lock_id).await;
        return Some(write_access.get_subscribers());
    }
}
