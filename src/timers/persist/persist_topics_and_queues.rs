use std::sync::Arc;

use crate::{app::AppContext, topics::Topic};

pub async fn save(app: Arc<AppContext>) -> Vec<Arc<Topic>> {
    let topics = app.topic_list.get_all().await;
    let mut topics_snapshots = Vec::new();

    for topic in &topics {
        topics_snapshots.push(topic.get_topic_snapshot().await);
    }

    let result = app.topics_and_queues_repo.save(topics_snapshots).await;

    if let Err(err) = result {
        app.logs.add_error(
            None,
            crate::app::logs::SystemProcess::TcpSocket,
            "persist::sync_topics_and_queues".to_string(),
            "Failed to save topics and queues snapshot".to_string(),
            Some(format!("{:?}", err)),
        );
    }

    topics
}
