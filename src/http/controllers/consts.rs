use my_http_server::middlewares::controllers::documentation::{
    HttpInputParameter, HttpParameterInputSource, HttpParameterType,
};

pub const AUTH_HEADER_NAME: &str = "authorization";

pub fn get_auth_header_description() -> HttpInputParameter {
    HttpInputParameter {
        name: "Authorization".to_string(),
        param_type: HttpParameterType::String,
        description: "Session, issued by greeting method".to_string(),
        source: HttpParameterInputSource::Header,
        required: true,
    }
}
