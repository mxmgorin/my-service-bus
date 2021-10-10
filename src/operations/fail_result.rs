use std::sync::Arc;

use crate::{queue_subscribers::SubscriberId, topics::Topic};

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

#[inline]
pub fn into_topic_result(
    src: Option<Arc<Topic>>,
    topic_id: &str,
) -> Result<Arc<Topic>, OperationFailResult> {
    match src {
        Some(topic) => Ok(topic),
        None => Err(OperationFailResult::TopicNotFound {
            topic_id: topic_id.to_string(),
        }),
    }
}
