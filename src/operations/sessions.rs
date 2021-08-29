use std::sync::Arc;

use crate::{app::AppContext, sessions::MyServiceBusSession};

pub async fn disconnect(app: &AppContext, id: i64) -> Option<Arc<MyServiceBusSession>> {
    let session = app.sessions.remove(&id).await?;

    let session_name = session.get_name().await;
    println!("We have a disconnect. Session: {}", session_name);

    let subscribers = session.disconnect().await;

    if subscribers.is_none() {
        return Some(session);
    }

    let subscribers = subscribers.unwrap();

    for subscriber in subscribers.values() {
        println!(
            "Sesision {} has a subscriber {}->{}",
            session_name, subscriber.topic_id, subscriber.queue_id
        );
    }

    for (subscriber_id, subscriber_data) in &subscribers {
        let topic = app.topic_list.get(subscriber_data.topic_id.as_str()).await;

        if let Some(topic) = topic {
            let queue = topic.queues.get(subscriber_data.queue_id.as_str()).await;

            if let Some(queue) = queue {
                let mut write_access = queue.data.write().await;

                let result = crate::operations::subscriber::unsubscribe(
                    session.as_ref(),
                    &mut write_access,
                    *subscriber_id,
                )
                .await;

                if let Err(err) = result {
                    app.logs
                        .add_error(
                            None,
                            crate::app::logs::SystemProcess::TcpSocket,
                            "operations::disconnect".to_string(),
                            format!(
                                "Can not unsubscriber subscriber {}. Data {}",
                                subscriber_id,
                                subscriber_data.to_string()
                            ),
                            Some(format!("{:?}", err)),
                        )
                        .await;
                }
            }
        }
    }

    return Some(session);
}
