use my_http_server_controllers::controllers::documentation::{
    data_types::HttpDataType, out_results::HttpResult,
};

pub fn empty(description: &str) -> HttpResult {
    HttpResult {
        http_code: 202,
        nullable: true,
        description: description.to_string(),
        data_type: HttpDataType::None,
    }
}

pub fn topic_or_queue_not_found() -> HttpResult {
    HttpResult {
        http_code: 404,
        nullable: false,
        description: "Topic or Queue is not found".to_string(),
        data_type: HttpDataType::as_string(),
    }
}

pub fn session_is_not_found() -> HttpResult {
    HttpResult {
        http_code: 404,
        nullable: false,
        description: "Topic or Queue is not found".to_string(),
        data_type: HttpDataType::as_string(),
    }
}
