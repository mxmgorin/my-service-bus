use std::sync::Arc;

use my_service_bus_tcp_shared::MessageToPublishTcpContract;

use crate::{app::AppContext, sessions::MyServiceBusSession, topics::Topic};

use super::OperationFailResult;

pub async fn create_topic_if_not_exists(
    app: Arc<AppContext>,
    session: Option<&MyServiceBusSession>,
    topic_id: &str,
) -> Arc<Topic> {
    let topic = app.topic_list.add_if_not_exists(topic_id.to_string()).await;

    tokio::task::spawn(crate::timers::persist::persist_topics_and_queues::save(
        app.clone(),
    ));
    tokio::task::spawn(crate::timers::persist::save_messages_for_topic(
        app,
        topic.clone(),
    ));

    {
        let mut topic_data = topic.get_access("create_topic_if_not_exists").await;

        if let Some(session) = session {
            topic_data.set_publisher_as_active(session.id);
        }
    }

    return topic;
}

pub async fn publish(
    app: Arc<AppContext>,
    topic_id: String,
    messages: Vec<MessageToPublishTcpContract>,
    persist_immediately: bool,
    session: Arc<MyServiceBusSession>,
) -> Result<(), OperationFailResult> {
    if app.states.is_shutting_down() {
        return Err(OperationFailResult::ShuttingDown);
    }

    let topic = app.topic_list.get(topic_id.as_str()).await;

    if topic.is_none() {
        if app.auto_create_topic_on_publish {
            app.topic_list.add_if_not_exists(topic_id).await;
        } else {
            return Err(OperationFailResult::TopicNotFound {
                topic_id: topic_id.to_string(),
            });
        }
    }

    let topic = topic.unwrap();

    let mut topic_data = topic.get_access("publish").await;

    let messages_count = messages.len();

    topic_data.publish_messages(session.id, messages);

    topic_data.metrics.update_topic_metrics(messages_count);

    if persist_immediately {
        tokio::task::spawn(crate::timers::persist::save_messages_for_topic(
            app.clone(),
            topic.clone(),
        ));
    }

    super::delivery::try_to_deliver(&app, &topic, &mut topic_data);
    Ok(())
}
