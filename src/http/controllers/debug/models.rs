use my_http_server_swagger::MyHttpInput;

#[derive(Debug, MyHttpInput)]
pub struct EnableDebugInputModel<'s> {
    #[http_query(name = "topicId"; description = "Id of topic")]
    pub topic_id: &'s str,
    #[http_query(name = "queueId"; description = "Id of queue")]
    pub queue_id: &'s str,
}

#[derive(Debug, MyHttpInput)]
pub struct GetOnDeliveryInputModel<'s> {
    #[http_query(name = "topicId"; description = "Id of topic")]
    pub topic_id: &'s str,
    #[http_query(name = "queueId"; description = "Id of queue")]
    pub queue_id: &'s str,
    #[http_query(name = "subscriberId"; description = "Id of subscriber")]
    pub subscriber_id: i64,
}
