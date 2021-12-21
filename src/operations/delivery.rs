use std::sync::Arc;

use my_service_bus_shared::page_id::PageId;
use my_service_bus_tcp_shared::DeliveryPackageBuilder;

use crate::{
    app::AppContext,
    queues::delivery_iterator::DeliveryIterator,
    tcp::tcp_server::ConnectionId,
    topics::{Topic, TopicData},
};

pub fn try_to_deliver(app: Arc<AppContext>, topic: &Arc<Topic>, topic_data: &mut TopicData) {
    while let Some(mut delivery_iterator) = topic_data.get_delivery_iterator(app.max_delivery_size)
    {
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
                            app.clone(),
                            &mut delivery_iterator,
                            &mut delivery_package_builder,
                            session_id,
                        );
                    } else {
                        delivery_iterator.subscriber.cancel_the_rent();
                    }

                    load_page_data(app, topic.clone(), page_id);
                    return;
                }
            }
        }

        if delivery_package_builder.len() > 0 {
            deliver_messages(
                app.clone(),
                &mut delivery_iterator,
                &mut delivery_package_builder,
                session_id,
            );
        } else {
            delivery_iterator.subscriber.cancel_the_rent();
        }
    }
}

fn deliver_messages(
    app: Arc<AppContext>,
    delivery_iterator: &mut DeliveryIterator,
    delivery_package_builder: &mut DeliveryPackageBuilder,
    session_id: ConnectionId,
) {
    delivery_iterator
        .subscriber
        .set_messages_on_delivery(&delivery_package_builder.ids);

    delivery_iterator.subscriber.metrics.set_started_delivery();

    let tcp_packet = delivery_package_builder.build();

    tokio::spawn(async move {
        crate::operations::sessions::send_package(&app, session_id, tcp_packet).await;
    });
}

fn load_page_data(app: Arc<AppContext>, topic: Arc<Topic>, page_id: PageId) {
    tokio::spawn(async move {
        crate::operations::page_loader::load_full_page_to_cache(
            topic.clone(),
            &app.messages_pages_repo,
            Some(app.logs.as_ref()),
            page_id,
        )
        .await;
        let mut topic_data = topic.data.lock().await;
        try_to_deliver(app, &topic, &mut topic_data);
    });
}
