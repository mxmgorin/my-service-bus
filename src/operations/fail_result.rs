use std::sync::Arc;

use crate::{
    subscribers::{Subscriber, SubscriberId},
    topics::Topic,
};

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

/*
pub fn into_queue_result(
    src: Option<Arc<TopicQueue>>,
    queue_id: &str,
) -> Result<Arc<TopicQueue>, OperationFailResult> {
    match src {
        Some(queue) => Ok(queue),
        None => Err(OperationFailResult::QueueNotFound {
            queue_id: queue_id.to_string(),
        }),
    }
}

pub fn into_subscriber_result(
    src: Option<&Subscriber>,
    id: SubscriberId,
) -> Result<&Subscriber, OperationFailResult> {
    match src {
        Some(subscriber) => Ok(subscriber),
        None => Err(OperationFailResult::SubscriberNotFound { id }),
    }
}
*/

pub fn into_subscriber_result_mut(
    src: Option<&mut Subscriber>,
    id: SubscriberId,
) -> Result<&mut Subscriber, OperationFailResult> {
    match src {
        Some(subscriber) => Ok(subscriber),
        None => Err(OperationFailResult::SubscriberNotFound { id }),
    }
}
