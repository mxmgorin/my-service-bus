use std::{sync::Arc, time::Duration};

use super::AppContext;

pub async fn execute(app: Arc<AppContext>) {
    empty_persistence_queues(app.clone()).await;
    make_last_topcis_and_queues_persist(app.clone()).await;
    app.states.set_shutted_down();
}

async fn empty_persistence_queues(app: Arc<AppContext>) {
    let duration = Duration::from_millis(500);
    for topic in app.topic_list.get_all().await {
        let msgs_to_persist = {
            let topic_data = topic.get_access("empty_persistence_queues").await;
            topic_data.pages.get_persist_queue_size()
        };

        while msgs_to_persist > 0 {
            println!(
                "Topic {} has {} messages to persist. Waiting 0.5 sec",
                topic.topic_id, msgs_to_persist
            );

            crate::timers::persist::save_messages_for_topic(app.clone(), topic.clone()).await;

            tokio::time::sleep(duration).await;
        }

        println!("Topic {} has no messages to persist.", topic.topic_id);
    }
}

async fn make_last_topcis_and_queues_persist(app: Arc<AppContext>) {
    println!("Making final topics and queues snapshot save");
    crate::timers::persist::persist_topics_and_queues::save(app).await;
    println!("Final topics and queues snapshot save is done");
}
