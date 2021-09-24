use std::{collections::HashMap, sync::Arc};

use my_service_bus_tcp_shared::ConnectionAttributes;
use tokio::{
    io::{AsyncWriteExt, WriteHalf},
    net::TcpStream,
};

use crate::{app::AppContext, subscribers::SubscriberId};

use super::{my_sb_session_subscriber_data::MySbSessionSubscriberData, MySbSessionStatistic};

pub struct MyServiceBusSessionData {
    pub name: Option<String>,
    pub client_version: Option<String>,

    pub subscribers: HashMap<SubscriberId, MySbSessionSubscriberData>,

    pub attr: ConnectionAttributes,

    pub tcp_stream: Option<WriteHalf<TcpStream>>,

    pub statistic: MySbSessionStatistic,

    pub app: Arc<AppContext>,

    pub logged_send_error_on_disconnected: i32,
}

impl MyServiceBusSessionData {
    pub fn new(tcp_stream: WriteHalf<TcpStream>, app: Arc<AppContext>) -> Self {
        Self {
            name: None,
            client_version: None,
            subscribers: HashMap::new(),
            attr: ConnectionAttributes::new(),
            tcp_stream: Some(tcp_stream),
            statistic: MySbSessionStatistic::new(),
            app,
            logged_send_error_on_disconnected: 0,
        }
    }

    pub fn is_disconnected(&self) -> bool {
        return self.tcp_stream.is_none();
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

    pub async fn send(&mut self, buf: &[u8]) -> Result<(), String> {
        match self.tcp_stream.as_mut() {
            Some(tcp_stream) => {
                let result = tcp_stream.write_all(buf).await;

                if let Err(err) = result {
                    return Err(format!(
                        "Can not send to the socket {:?}. Err:{}",
                        self.name, err
                    ));
                } else {
                    self.statistic.increase_written_size(buf.len()).await;
                    return Ok(());
                }
            }
            None => {
                return Err(format!("Socket {:?} is disconnected", self.name));
            }
        }
    }

    pub async fn disconnect(&mut self) {
        if self.tcp_stream.is_none() {
            return;
        }

        let mut tcp_stream = None;

        std::mem::swap(&mut tcp_stream, &mut self.tcp_stream);

        self.statistic.disconnected = true;

        let result = tcp_stream.unwrap().shutdown().await;

        if let Err(err) = result {
            return println!(
                "Can nod disconnect tcp socket{:?}. Err: {:?}",
                self.name, err
            );
        }
    }

    pub fn set_on_delivery_flag(&mut self, subscriber_id: SubscriberId) {
        let subscriber = self.statistic.subscribers.get_mut(&subscriber_id);

        if let Some(subscriber) = subscriber {
            subscriber.active = 2;
        }
    }
}
