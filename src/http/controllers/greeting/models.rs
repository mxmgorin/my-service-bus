use my_http_server_swagger::{MyHttpInput, MyHttpObjectStructure};
use serde::{Deserialize, Serialize};

#[derive(Debug, MyHttpInput)]
pub struct GreetingInputModel {
    #[http_form(description = "Name of application")]
    pub name: String,

    #[http_form(description = "Version of application")]
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct GreetingJsonResult {
    pub session: String,
}

#[derive(Debug, MyHttpInput)]
pub struct PingInputModel {
    #[http_header(description = "Http session")]
    pub authorization: String,
}
