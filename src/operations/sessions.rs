use std::sync::Arc;

use crate::{app::AppContext, sessions::MyServiceBusSession};

pub async fn disconnect(process_id: i64, app: &AppContext, session: Arc<MyServiceBusSession>) {
    let session_name = session.get_name(process_id).await;

    let subscribers = session.disconnect(process_id).await;

    if subscribers.is_none() {
        return;
    }

    let subscribers = subscribers.unwrap();

    for (subscriber_id, subscriber_data) in &subscribers {
        println!(
            "Sesision {} has a subscriber {}->{}",
            session_name, subscriber_data.topic_id, subscriber_data.queue_id
        );

        let topic = app.topic_list.get(subscriber_data.topic_id.as_str()).await;

        if let Some(topic) = topic {
            let queue = topic.queues.get(subscriber_data.queue_id.as_str()).await;

            if let Some(queue) = queue {
                let mut write_access = queue.data.write().await;

                let result = crate::operations::subscriber::unsubscribe(
                    process_id,
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
}
