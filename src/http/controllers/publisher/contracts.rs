use std::collections::HashMap;

use my_http_server::{HttpFailResult, WebContentType};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageKeyValueJsonModel {
    pub key: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageToPublishJsonModel {
    pub headers: Option<Vec<MessageKeyValueJsonModel>>,
    pub base64_message: String,
}

impl MessageToPublishJsonModel {
    pub fn get_headers(&mut self) -> Option<HashMap<String, String>> {
        let mut result = None;

        std::mem::swap(&mut self.headers, &mut result);

        let src = result?;
        let mut result = HashMap::new();

        for itm in src {
            result.insert(itm.key, itm.value);
        }

        Some(result)
    }

    pub fn get_content(&self) -> Result<Vec<u8>, HttpFailResult> {
        match base64::decode(self.base64_message.as_str()) {
            Ok(bytes) => Ok(bytes),
            Err(err) => Err(HttpFailResult {
                content_type: WebContentType::Text,
                status_code: 400,
                content: format!("Can not convert content from Base64. Err: Err{}", err)
                    .into_bytes(),
                write_telemetry: false,
            }),
        }
    }
}
