use my_http_server::middlewares::controllers::documentation::{
    data_types::{HttpDataProperty, HttpDataType},
    in_parameters::{HttpInputParameter, HttpParameterInputSource},
    out_results::HttpResult,
};

pub const AUTH_HEADER_NAME: &str = "authorization";

pub fn get_auth_header_description() -> HttpInputParameter {
    HttpInputParameter {
        data_property: HttpDataProperty::new("Authorization", HttpDataType::as_string(), true),

        description: "Session, issued by greeting method".to_string(),
        source: HttpParameterInputSource::Header,
        required: true,
    }
}

pub fn get_topic_id_parameter() -> HttpInputParameter {
    HttpInputParameter {
        data_property: HttpDataProperty::new("topicId", HttpDataType::as_string(), true),

        description: "Id of topic".to_string(),
        source: HttpParameterInputSource::Query,
        required: true,
    }
}

pub fn get_queue_id_parameter() -> HttpInputParameter {
    HttpInputParameter {
        data_property: HttpDataProperty::new("queueId", HttpDataType::as_string(), true),

        description: "Id of queue".to_string(),
        source: HttpParameterInputSource::Query,
        required: true,
    }
}

pub fn get_message_id_parameter() -> HttpInputParameter {
    HttpInputParameter {
        data_property: HttpDataProperty::new("messageId", HttpDataType::as_long(), true),

        description: "Id of message".to_string(),
        source: HttpParameterInputSource::Query,
        required: true,
    }
}

pub fn get_connection_id_parameter() -> HttpInputParameter {
    HttpInputParameter {
        data_property: HttpDataProperty::new("connectionId", HttpDataType::as_long(), true),

        description: "Id of connection".to_string(),
        source: HttpParameterInputSource::Query,
        required: true,
    }
}

pub fn get_empty_result() -> Vec<HttpResult> {
    vec![HttpResult {
        http_code: 202,
        nullable: false,
        description: "Session description".to_string(),
        data_type: HttpDataType::None,
    }]
}

pub fn get_text_result() -> Vec<HttpResult> {
    vec![HttpResult {
        http_code: 200,
        nullable: false,
        description: "Session description".to_string(),
        data_type: HttpDataType::as_string(),
    }]
}
