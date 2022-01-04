use std::sync::Arc;

use my_service_bus_tcp_shared::DeliveryPackageBuilder;

use crate::{
    queues::delivery_iterator::DeliveryIterator,
    sessions::SessionId,
    topics::{Topic, TopicData},
};

use super::DeliveryDependecies;

pub fn try_to_deliver<TDeliveryDependecies: DeliveryDependecies>(
    delivery: &TDeliveryDependecies,
    topic: &Arc<Topic>,
    topic_data: &mut TopicData,
) {
    let max_delivery_size = delivery.get_max_delivery_size();
    while let Some(mut delivery_iterator) = topic_data.get_delivery_iterator(max_delivery_size) {
        let mut delivery_package_builder = DeliveryPackageBuilder::new(
            delivery_iterator.topic_id,
            delivery_iterator.queue_id,
            delivery_iterator.subscriber.id,
            delivery_iterator.subscriber.delivery_packet_version,
        );

        let session_id = delivery_iterator.subscriber.session_id;

        while let Some(next_message) = delivery_iterator.next() {
            match next_message {
                crate::queues::delivery_iterator::NextMessageResult::Value {
                    content,
                    attempt_no,
                } => {
                    if delivery_package_builder.payload_size == 0
                        || delivery_package_builder.payload_size + content.content.len()
                            <= max_delivery_size
                    {
                        delivery_package_builder.add_message(content, attempt_no);
                    } else {
                        break;
                    }
                }
                crate::queues::delivery_iterator::NextMessageResult::LoadDataRequired(page_id) => {
                    if delivery_package_builder.len() > 0 {
                        deliver_messages(
                            delivery,
                            &mut delivery_iterator,
                            &mut delivery_package_builder,
                            session_id,
                        );
                    } else {
                        delivery_iterator.subscriber.cancel_the_rent();
                    }

                    delivery.load_page(topic.clone(), page_id);
                    return;
                }
            }
        }

        if delivery_package_builder.len() > 0 {
            deliver_messages(
                delivery,
                &mut delivery_iterator,
                &mut delivery_package_builder,
                session_id,
            );
        } else {
            delivery_iterator.subscriber.cancel_the_rent();
        }
    }
}

fn deliver_messages<TDeliveryDependecies: DeliveryDependecies>(
    delivery: &TDeliveryDependecies,
    delivery_iterator: &mut DeliveryIterator,
    delivery_package_builder: &mut DeliveryPackageBuilder,
    session_id: SessionId,
) {
    delivery_iterator
        .subscriber
        .set_messages_on_delivery(&delivery_package_builder.ids);

    delivery_iterator.subscriber.metrics.set_started_delivery();

    let tcp_packet = delivery_package_builder.build();

    delivery.send_package(session_id, tcp_packet);
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use my_service_bus_shared::{
        messages_page::{MessagesPage, MessagesPageRestoreSnapshot},
        queue::TopicQueueType,
        MySbMessageContent,
    };
    use rust_extensions::date_time::DateTimeAsMicroseconds;

    use super::super::delivery_dependency_mock::DeliveryDependeciesMock;
    use crate::queue_subscribers::QueueSubscriberDeliveryState;

    use super::*;

    #[tokio::test]
    async fn test_two_publish_two_delivery() {
        const TOPIC_NAME: &str = "TestTopic";
        const QUEUE_NAME: &str = "TestQueue";
        const SUBSCRIBER_ID: i64 = 15;
        const SESSION_ID: SessionId = 13;
        const DELIVERY_SIZE: usize = 16;

        let topic = Arc::new(Topic::new(TOPIC_NAME.to_string(), 0));

        let mut topic_data = topic.get_access("test_two_publish_two_delivery").await;

        {
            let queue = topic_data.queues.add_queue_if_not_exists(
                TOPIC_NAME.to_string(),
                QUEUE_NAME.to_string(),
                TopicQueueType::Permanent,
            );

            queue
                .subscribers
                .subscribe(
                    SUBSCRIBER_ID,
                    TOPIC_NAME.to_string(),
                    QUEUE_NAME.to_string(),
                    SESSION_ID,
                    1,
                )
                .unwrap();
        }

        let messages = vec![vec![0u8, 1u8, 2u8], vec![3u8, 4u8, 5u8]];
        topic_data.publish_messages(SESSION_ID, messages);

        let delivery_dependecies = DeliveryDependeciesMock::new(DELIVERY_SIZE);

        try_to_deliver(&delivery_dependecies, &topic, &mut topic_data);

        let sent_packets = delivery_dependecies.get_sent_packets();

        assert_eq!(sent_packets.len(), 1);

        assert_eq!(sent_packets[0].0, SESSION_ID);

        let queue = topic_data.queues.get(QUEUE_NAME).unwrap();

        let subscriber = queue.subscribers.get_by_id(SUBSCRIBER_ID).unwrap();

        if let QueueSubscriberDeliveryState::OnDelivery(data) = &subscriber.delivery_state {
            assert_eq!(data.bucket.ids.len(), 2);
        } else {
            panic!("Should not be here");
        }
    }

    #[tokio::test]
    async fn test_two_publish_one_delivery() {
        const TOPIC_NAME: &str = "TestTopic";
        const QUEUE_NAME: &str = "TestQueue";
        const SUBSCRIBER_ID: i64 = 15;
        const SESSION_ID: SessionId = 13;
        const DELIVERY_SIZE: usize = 4;

        let topic = Arc::new(Topic::new(TOPIC_NAME.to_string(), 0));

        let mut topic_data = topic.get_access("test_two_publish_one_delivery").await;

        {
            let queue = topic_data.queues.add_queue_if_not_exists(
                TOPIC_NAME.to_string(),
                QUEUE_NAME.to_string(),
                TopicQueueType::Permanent,
            );

            queue
                .subscribers
                .subscribe(
                    SUBSCRIBER_ID,
                    TOPIC_NAME.to_string(),
                    QUEUE_NAME.to_string(),
                    SESSION_ID,
                    1,
                )
                .unwrap();
        }

        let messages = vec![vec![0u8, 1u8, 2u8], vec![3u8, 4u8, 5u8]];
        topic_data.publish_messages(SESSION_ID, messages);

        let delivery_dependecies = DeliveryDependeciesMock::new(DELIVERY_SIZE);

        try_to_deliver(&delivery_dependecies, &topic, &mut topic_data);

        let sent_packets = delivery_dependecies.get_sent_packets();

        assert_eq!(sent_packets.len(), 1);

        assert_eq!(sent_packets[0].0, SESSION_ID);

        let queue = topic_data.queues.get(QUEUE_NAME).unwrap();

        let subscriber = queue.subscribers.get_by_id(SUBSCRIBER_ID).unwrap();

        if let QueueSubscriberDeliveryState::OnDelivery(data) = &subscriber.delivery_state {
            assert_eq!(data.bucket.ids.len(), 1);
        } else {
            panic!("Should not be here");
        }
    }

    #[tokio::test]
    async fn test_with_first_not_loaded_message() {
        const TOPIC_NAME: &str = "TestTopic";
        const QUEUE_NAME: &str = "TestQueue";
        const SUBSCRIBER_ID: i64 = 15;
        const SESSION_ID: SessionId = 13;
        const DELIVERY_SIZE: usize = 4;

        let topic = Arc::new(Topic::new(TOPIC_NAME.to_string(), 0));

        let mut topic_data = topic.get_access("test_with_first_not_loaded_message").await;

        {
            let queue = topic_data.queues.add_queue_if_not_exists(
                TOPIC_NAME.to_string(),
                QUEUE_NAME.to_string(),
                TopicQueueType::Permanent,
            );

            queue
                .subscribers
                .subscribe(
                    SUBSCRIBER_ID,
                    TOPIC_NAME.to_string(),
                    QUEUE_NAME.to_string(),
                    SESSION_ID,
                    1,
                )
                .unwrap();
        }

        //Restoring Page with  #0 - NotLoaded, #1 - Loaded;
        {
            let mut messages = HashMap::new();

            messages.insert(
                1,
                MySbMessageContent::new(1, vec![0u8, 1u8, 2u8], DateTimeAsMicroseconds::now()),
            );

            let restore_snapshot =
                MessagesPageRestoreSnapshot::new_with_messages(0, 1, 1, messages);

            let page = MessagesPage::restore(restore_snapshot);

            topic_data.pages.restore_page(page);
            let queue = topic_data.queues.get_mut(QUEUE_NAME).unwrap();

            queue.queue.enqueue(0);
            queue.queue.enqueue(1);
        }

        let delivery_dependecies = DeliveryDependeciesMock::new(DELIVERY_SIZE);

        try_to_deliver(&delivery_dependecies, &topic, &mut topic_data);

        {
            let sent_packets = delivery_dependecies.get_sent_packets();
            assert_eq!(sent_packets.len(), 0);
        }

        let queue = topic_data.queues.get(QUEUE_NAME).unwrap();

        let subscriber = queue.subscribers.get_by_id(SUBSCRIBER_ID).unwrap();

        if let QueueSubscriberDeliveryState::ReadyToDeliver = &subscriber.delivery_state {
            let (topic, page_id) = delivery_dependecies.get_load_page_event_data();
            assert_eq!(TOPIC_NAME, topic.topic_id.as_str());
            assert_eq!(page_id, 0);
        } else {
            panic!("Should not be here");
        }
    }

    #[tokio::test]
    async fn test_with_all_messages_missing() {
        const TOPIC_NAME: &str = "TestTopic";
        const QUEUE_NAME: &str = "TestQueue";
        const SUBSCRIBER_ID: i64 = 15;
        const SESSION_ID: SessionId = 13;
        const DELIVERY_SIZE: usize = 4;

        let topic = Arc::new(Topic::new(TOPIC_NAME.to_string(), 0));

        let mut topic_data = topic.get_access("test_with_all_messages_missing").await;

        {
            let queue = topic_data.queues.add_queue_if_not_exists(
                TOPIC_NAME.to_string(),
                QUEUE_NAME.to_string(),
                TopicQueueType::Permanent,
            );

            queue
                .subscribers
                .subscribe(
                    SUBSCRIBER_ID,
                    TOPIC_NAME.to_string(),
                    QUEUE_NAME.to_string(),
                    SESSION_ID,
                    1,
                )
                .unwrap();
        }

        //Restoring Page with  #0 - NotLoaded, #1 - Loaded;
        {
            let restore_snapshot = MessagesPageRestoreSnapshot::new(0, 0, 1);

            let page = MessagesPage::restore(restore_snapshot);

            topic_data.pages.restore_page(page);
            let queue = topic_data.queues.get_mut(QUEUE_NAME).unwrap();

            queue.queue.enqueue(0);
            queue.queue.enqueue(1);
        }

        let delivery_dependecies = DeliveryDependeciesMock::new(DELIVERY_SIZE);

        try_to_deliver(&delivery_dependecies, &topic, &mut topic_data);

        {
            let sent_packets = delivery_dependecies.get_sent_packets();
            assert_eq!(sent_packets.len(), 0);
        }

        let queue = topic_data.queues.get(QUEUE_NAME).unwrap();

        let subscriber = queue.subscribers.get_by_id(SUBSCRIBER_ID).unwrap();

        if let QueueSubscriberDeliveryState::ReadyToDeliver = &subscriber.delivery_state {
            assert_eq!(0, queue.queue.len());
        } else {
            panic!("Should not be here");
        }
    }
}
