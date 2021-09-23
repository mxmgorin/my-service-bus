use std::collections::HashMap;

use my_service_bus_tcp_shared::ConnectionAttributes;
use tokio::{
    io::{AsyncWriteExt, WriteHalf},
    net::TcpStream,
};

use crate::{app::logs::Logs, subscribers::SubscriberId};

use super::{my_sb_session_subscriber_data::MySbSessionSubscriberData, MySbSessionStatistic};

pub struct MyServiceBusSessionData {
    pub name: Option<String>,
    pub client_version: Option<String>,

    subscribers: HashMap<SubscriberId, MySbSessionSubscriberData>,

    pub attr: ConnectionAttributes,

    pub tcp_stream: Option<WriteHalf<TcpStream>>,

    pub statistic: MySbSessionStatistic,
}

impl MyServiceBusSessionData {
    pub fn new(tcp_stream: WriteHalf<TcpStream>) -> Self {
        Self {
            name: None,
            client_version: None,
            subscribers: HashMap::new(),
            attr: ConnectionAttributes::new(),
            tcp_stream: Some(tcp_stream),
            statistic: MySbSessionStatistic::new(),
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

    pub async fn send(&mut self, buf: &[u8], logs: &Logs) {
        match self.tcp_stream.as_mut() {
            Some(tcp_stream) => {
                let result = tcp_stream.write_all(buf).await;

                if let Err(err) = result {
                    logs.add_error(
                        None,
                        crate::app::logs::SystemProcess::TcpSocket,
                        "MyServiceBusSession.send".to_string(),
                        format!("Can not send to the socket {:?}", self.name),
                        Some(format!("{:?}", err)),
                    )
                    .await;

                    self.disconnect(logs).await;
                } else {
                    self.statistic.increase_written_size(buf.len()).await;
                }
            }
            None => {
                logs.add_error(
                    None,
                    crate::app::logs::SystemProcess::TcpSocket,
                    "MyServiceBusSession.send".to_string(),
                    format!("Socket {:?} is disconnected", self.name),
                    None,
                )
                .await;
            }
        }
    }

    pub async fn disconnect(&mut self, logs: &Logs) {
        if self.tcp_stream.is_none() {
            return;
        }

        let mut tcp_stream = None;

        std::mem::swap(&mut tcp_stream, &mut self.tcp_stream);

        let result = tcp_stream.unwrap().shutdown().await;

        if let Err(err) = result {
            logs.add_error(
                None,
                crate::app::logs::SystemProcess::TcpSocket,
                "my_sb_session_data.disconnect()".to_string(),
                "Disconnect Error".to_string(),
                Some(format!("{:?}", err)),
            )
            .await
        }
    }
}
