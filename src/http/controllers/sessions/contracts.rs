use my_http_server_swagger::MyHttpInput;

#[derive(MyHttpInput)]
pub struct DeleteSessionInputContract {
    #[http_query(name = "connectionId"; description = "Id of connection")]
    pub connection_id: i64,
}
