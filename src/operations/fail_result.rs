use crate::queue_subscribers::SubscriberId;

#[derive(Debug)]
pub enum OperationFailResult {
    TopicNotFound { topic_id: String },
    QueueNotFound { queue_id: String },
    SubscriberNotFound { id: SubscriberId },
    SessionIsDisconnected,
    InvalidProtobufPayload(String),
    PersistenceError(String),
    TonicError(tonic::Status),
    Other(String),
    ShuttingDown,
}
