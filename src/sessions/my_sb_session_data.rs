use std::sync::Arc;

use my_service_bus_tcp_shared::ConnectionAttributes;
use tokio::{
    io::{AsyncWriteExt, WriteHalf},
    net::TcpStream,
};

use crate::app::AppContext;

use super::{MySbSessionMetrics, SessionOperationError};

pub struct ConnectedState {
    tcp_stream: WriteHalf<TcpStream>,
}

pub struct MyServiceBusSessionData {
    pub name: Option<String>,
    pub client_version: Option<String>,

    pub connected_state: Option<ConnectedState>,

    pub attr: ConnectionAttributes,

    pub statistic: MySbSessionMetrics,

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
            statistic: MySbSessionMetrics::new(),
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

    fn get_connected_state_mut(&mut self) -> Result<&mut ConnectedState, SessionOperationError> {
        match &mut self.connected_state {
            Some(state) => Ok(state),
            None => Err(SessionOperationError::Disconnected),
        }
    }

    pub async fn send(&mut self, buf: &[u8]) -> Result<(), SessionOperationError> {
        let connected_state = self.get_connected_state_mut()?;

        let result = connected_state.tcp_stream.write_all(buf).await;

        if let Err(err) = result {
            println!("Could not send to the connection. Disconnecting Session");
            let disconnect_result = self.disconnect().await;

            if let Some(_) = disconnect_result {
                return Err(SessionOperationError::JustDisconnected);
            };

            return Err(SessionOperationError::CanNotSendOperationToSocket(format!(
                "Can not send to the socket {:?}. Err:{}",
                self.name, err
            )));
        } else {
            self.statistic.increase_written_size(buf.len()).await;
        }

        Ok(())
    }

    pub async fn disconnect(&mut self) -> Option<ConnectedState> {
        let mut connected_state = None;
        std::mem::swap(&mut connected_state, &mut self.connected_state);

        if connected_state.is_none() {
            return None;
        }

        let mut connected_state = connected_state.unwrap();
        self.statistic.disconnected = true;

        let result = connected_state.tcp_stream.shutdown().await;

        if let Err(err) = result {
            println!(
                "Error on diconnect session {:?}. Err: {:?}",
                self.get_name(),
                err
            );
        }

        Some(connected_state)
    }
}
