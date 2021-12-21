use std::sync::Arc;

use my_service_bus_tcp_shared::ConnectionAttributes;
use tokio::{
    io::{AsyncWriteExt, WriteHalf},
    net::TcpStream,
};

use crate::app::AppContext;

use super::MySbSessionMetrics;

pub struct ConnectedState {
    tcp_stream: WriteHalf<TcpStream>,
}

pub struct MyServiceBusSessionData {
    pub name: Option<String>,
    pub client_version: Option<String>,

    pub connected_state: Option<ConnectedState>,

    pub attr: ConnectionAttributes,

    pub metrics: MySbSessionMetrics,

    pub app: Arc<AppContext>,

    pub logged_send_error_on_disconnected: i32,
}

impl MyServiceBusSessionData {
    pub fn new(tcp_stream: WriteHalf<TcpStream>, app: Arc<AppContext>) -> Self {
        let connected_state = ConnectedState { tcp_stream };
        Self {
            name: None,
            client_version: None,
            connected_state: Some(connected_state),
            attr: ConnectionAttributes::new(),
            metrics: MySbSessionMetrics::new(),
            app,
            logged_send_error_on_disconnected: 0,
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

    pub async fn send(&mut self, buf: &[u8]) -> bool {
        if self.connected_state.is_none() {
            return false;
        }

        let connected_state = self.connected_state.as_mut().unwrap();

        let result = connected_state.tcp_stream.write_all(buf).await;

        if let Err(err) = result {
            println!(
                "Could not send to the connection {:?}. Err {}",
                self.name, err
            );
            self.disconnect().await;

            return false;
        } else {
            self.metrics.increase_written_size(buf.len());
            return true;
        }
    }

    pub async fn disconnect(&mut self) {
        if self.connected_state.is_none() {
            return;
        }

        let mut connected_state = None;
        std::mem::swap(&mut connected_state, &mut self.connected_state);

        let mut connected_state = connected_state.unwrap();
        self.metrics.disconnected = true;

        let result = connected_state.tcp_stream.shutdown().await;

        if let Err(err) = result {
            println!(
                "Error on diconnect session {:?}. Err: {:?}",
                self.get_name(),
                err
            );
        }
    }
}
