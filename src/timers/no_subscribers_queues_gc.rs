use crate::{app::AppContext, date_time::MyDateTime, topics::Topic};

pub async fn execute(app: &AppContext, topic: &Topic) {
    let queues = topic.get_all_queues().await;

    let now = MyDateTime::utc_now();

    for queue in queues {
        let gc_data = queue.get_gc_data().await;

        match gc_data.queue_type {
            my_service_bus_shared::TopicQueueType::DeleteOnDisconnect => {
                if gc_data.subscribers_amount == 0
                    && now.get_duration_from(gc_data.last_subscriber_disconnect)
                        > app.empty_queue_gc_timeout
                {
                    topic.delete_queue(queue.queue_id.as_str()).await;
                }
            }
            _ => {}
        }
    }
}
