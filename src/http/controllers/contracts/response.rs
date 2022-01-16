use my_http_server::middlewares::controllers::documentation::{
    data_types::{HttpDataType, HttpObjectStructure},
    out_results::HttpResult,
};

pub fn empty(description: &str) -> HttpResult {
    HttpResult {
        http_code: 202,
        nullable: true,
        description: description.to_string(),
        data_type: HttpDataType::None,
    }
}

pub fn object(description: &str) -> Vec<HttpResult> {
    vec![HttpResult {
        http_code: 200,
        nullable: true,
        description: description.to_string(),
        data_type: HttpDataType::Object(HttpObjectStructure::new("EmptyContract")),
    }]
}

pub fn empty_and_authorized(description: &str) -> Vec<HttpResult> {
    vec![
        HttpResult {
            http_code: 202,
            nullable: true,
            description: description.to_string(),
            data_type: HttpDataType::None,
        },
        unathorized_http_result(),
    ]
}

pub fn text(description: &str) -> Vec<HttpResult> {
    vec![HttpResult {
        http_code: 200,
        nullable: false,
        description: description.to_string(),
        data_type: HttpDataType::as_string(),
    }]
}

fn unathorized_http_result() -> HttpResult {
    HttpResult {
        http_code: 401,
        nullable: true,
        description: "Unauthorized request".to_string(),
        data_type: HttpDataType::None,
    }
}

pub fn topic_not_found() -> HttpResult {
    HttpResult {
        http_code: 404,
        nullable: false,
        description: "Topic not found".to_string(),
        data_type: HttpDataType::as_string(),
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
