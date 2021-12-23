use std::sync::Arc;

use my_service_bus_tcp_shared::DeliveryPackageBuilder;

use crate::{
    queues::delivery_iterator::DeliveryIterator,
    tcp::tcp_server::ConnectionId,
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
                    delivery_package_builder.add_message(content, attempt_no);
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
    session_id: ConnectionId,
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

    use std::sync::Mutex;

    use my_service_bus_shared::queue::TopicQueueType;
    use my_service_bus_tcp_shared::TcpContract;

    use super::*;

    struct DeliveryDependeciesMock {
        sent_packets: Mutex<Vec<(ConnectionId, TcpContract)>>,
    }

    impl DeliveryDependeciesMock {
        pub fn new() -> Self {
            Self {
                sent_packets: Mutex::new(Vec::new()),
            }
        }
    }

    impl DeliveryDependecies for DeliveryDependeciesMock {
        fn get_max_delivery_size(&self) -> usize {
            16
        }

        fn send_package(&self, session_id: ConnectionId, tcp_packet: TcpContract) {
            let mut sent_packets = self.sent_packets.lock().unwrap();
            sent_packets.push((session_id, tcp_packet));
        }

        fn load_page(&self, topic: Arc<Topic>, page_id: my_service_bus_shared::page_id::PageId) {
            todo!()
        }
    }

    #[tokio::test]
    async fn test_one_publish_one_delivery() {
        const TOPIC_NAME: &str = "TestTopic";
        const QUEUE_NAME: &str = "TestQueue";
        const SUBSCRIBER_ID: i64 = 15;
        const SESSION_ID: ConnectionId = 13;

        let topic = Arc::new(Topic::new(TOPIC_NAME.to_string(), 0));

        let mut topic_data = topic.data.lock().await;

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

        let messages = vec![vec![0u8, 1u8, 2u8]];
        topic_data.publish_messages(SESSION_ID, messages);

        let delivery_dependecies = DeliveryDependeciesMock::new();

        try_to_deliver(&delivery_dependecies, &topic, &mut topic_data);

        let sent_packets = delivery_dependecies.sent_packets.lock().unwrap();

        assert_eq!(sent_packets.len(), 1);

        assert_eq!(sent_packets[0].0, SESSION_ID);
    }
}
