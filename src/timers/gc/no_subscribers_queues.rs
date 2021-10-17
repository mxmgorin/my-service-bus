use std::sync::Arc;

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{app::AppContext, topics::Topic};

pub async fn execute(app: Arc<AppContext>, topic: Arc<Topic>) {
    let queues = topic.get_all_queues().await;

    let now = DateTimeAsMicroseconds::now();

    for queue in queues {
        let gc_data = queue.get_gc_data().await;

        match gc_data.queue_type {
            my_service_bus_shared::queue::TopicQueueType::DeleteOnDisconnect => {
                if gc_data.subscribers_amount == 0 {
                    let since_last_disconnect =
                        now.duration_since(gc_data.last_subscriber_disconnect);
                    println!("Detected DeleteOnDisconnect queue {}/{} with 0 subscribers. Last disconnect since {:?}", topic.topic_id, queue.queue_id, since_last_disconnect);

                    if now.duration_since(gc_data.last_subscriber_disconnect)
                        > app.empty_queue_gc_timeout
                    {
                        topic.delete_queue(queue.queue_id.as_str()).await;
                    }
                }
            }
            _ => {}
        }
    }
}
