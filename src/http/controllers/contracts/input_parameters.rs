use my_http_server::middlewares::controllers::documentation::{
    data_types::{HttpDataType, HttpField},
    in_parameters::{HttpInputParameter, HttpParameterInputSource},
};

pub const AUTH_HEADER_NAME: &str = "authorization";

pub fn auth_header() -> HttpInputParameter {
    HttpInputParameter {
        field: HttpField::new("Authorization", HttpDataType::as_string(), true),

        description: "Session, issued by greeting method".to_string(),
        source: HttpParameterInputSource::Header,
        required: true,
    }
}

pub fn topic_id() -> HttpInputParameter {
    HttpInputParameter {
        field: HttpField::new("topicId", HttpDataType::as_string(), true),

        description: "Id of topic".to_string(),
        source: HttpParameterInputSource::Query,
        required: true,
    }
}

pub fn queue_id() -> HttpInputParameter {
    HttpInputParameter {
        field: HttpField::new("queueId", HttpDataType::as_string(), true),

        description: "Id of queue".to_string(),
        source: HttpParameterInputSource::Query,
        required: true,
    }
}

pub fn subscriber_id() -> HttpInputParameter {
    HttpInputParameter {
        field: HttpField::new("subscriberId", HttpDataType::as_long(), true),

        description: "Id of subscriber".to_string(),
        source: HttpParameterInputSource::Query,
        required: true,
    }
}

pub fn message_id() -> HttpInputParameter {
    HttpInputParameter {
        field: HttpField::new("messageId", HttpDataType::as_long(), true),

        description: "Id of message".to_string(),
        source: HttpParameterInputSource::Query,
        required: true,
    }
}

pub fn connection_id() -> HttpInputParameter {
    HttpInputParameter {
        field: HttpField::new("connectionId", HttpDataType::as_long(), true),

        description: "Id of connection".to_string(),
        source: HttpParameterInputSource::Query,
        required: true,
    }
}
