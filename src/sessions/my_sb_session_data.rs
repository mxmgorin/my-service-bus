use std::sync::Arc;

use my_service_bus_tcp_shared::ConnectionAttributes;

use crate::app::AppContext;

pub struct MyServiceBusSessionData {
    pub name: Option<String>,
    pub client_version: Option<String>,

    pub attr: ConnectionAttributes,

    pub app: Arc<AppContext>,

    pub logged_send_error_on_disconnected: i32,
}

impl MyServiceBusSessionData {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self {
            name: None,
            client_version: None,
            attr: ConnectionAttributes::new(),
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
}
