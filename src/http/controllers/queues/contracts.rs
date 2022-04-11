use my_http_server_swagger::MyHttpInput;

#[derive(MyHttpInput)]
pub struct GetListOfQueuesInputContract<'s> {
    #[http_query(name="topicId"; description = "Id of topic")]
    pub topic_id: &'s str,
}

#[derive(MyHttpInput)]
pub struct DeleteQueueInputContract<'s> {
    #[http_query(name="topicId"; description = "Id of topic")]
    pub topic_id: &'s str,
    #[http_query(name="queueId"; description = "Id of queue")]
    pub queue_id: &'s str,
}

#[derive(MyHttpInput)]
pub struct SetQueueMessageIdInputContract<'s> {
    #[http_query(name="topicId"; description = "Id of topic")]
    pub topic_id: &'s str,
    #[http_query(name="queueId"; description = "Id of queue")]
    pub queue_id: &'s str,
    #[http_query(name="messageId"; description = "Message id")]
    pub message_id: i64,
}
