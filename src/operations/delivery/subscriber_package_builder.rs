use std::sync::Arc;

use my_service_bus_shared::{queue_with_intervals::QueueWithIntervals, MySbMessageContent};
use my_service_bus_tcp_shared::{PacketProtVer, TcpContract};

use crate::{queue_subscribers::SubscriberId, sessions::MyServiceBusSession, topics::Topic};

pub enum SendNewMessagesResult {
    Send {
        session: Arc<MyServiceBusSession>,
        tcp_contract: TcpContract,
        queue_id: String,
        messages_on_delivery: QueueWithIntervals,
    },
    NothingToSend {
        queue_id: String,
    },
}

pub struct SubscriberPackageBuilder {
    pub topic: Arc<Topic>,
    pub queue_id: String,
    pub payload: Option<Vec<u8>>,
    pub session: Arc<MyServiceBusSession>,
    pub subscriber_id: SubscriberId,
    messages_on_delivery: QueueWithIntervals,
    messages_count_position: usize,
    version: PacketProtVer,
}

impl SubscriberPackageBuilder {
    pub fn new(
        topic: Arc<Topic>,
        queue_id: String,
        session: Arc<MyServiceBusSession>,
        subscriber_id: SubscriberId,
        version: PacketProtVer,
    ) -> Self {
        Self {
            topic,
            queue_id,
            payload: None,
            subscriber_id,
            session,
            messages_on_delivery: QueueWithIntervals::new(),
            messages_count_position: 0,
            version,
        }
    }

    fn get_or_create_payload(&mut self) -> &mut Vec<u8> {
        if self.payload.is_none() {
            let mut payload = Vec::new();

            payload.push(my_service_bus_tcp_shared::tcp_message_id::NEW_MESSAGES);
            my_service_bus_tcp_shared::tcp_serializers::pascal_string::serialize(
                &mut payload,
                self.topic.topic_id.as_str(),
            );
            my_service_bus_tcp_shared::tcp_serializers::pascal_string::serialize(
                &mut payload,
                self.queue_id.as_str(),
            );
            my_service_bus_tcp_shared::tcp_serializers::i64::serialize(
                &mut payload,
                self.subscriber_id,
            );

            self.messages_count_position = payload.len();
            my_service_bus_tcp_shared::tcp_serializers::i32::serialize(&mut payload, 0);

            self.payload = Some(payload);
        }

        self.payload.as_mut().unwrap()
    }

    pub fn data_size(&self) -> usize {
        match &self.payload {
            Some(payload) => payload.len(),
            None => 0,
        }
    }

    pub fn add_message(&mut self, message_content: &MySbMessageContent, attempt_no: i32) {
        self.messages_on_delivery.enqueue(message_content.id);

        let version = self.version.clone();

        let payload = self.get_or_create_payload();

        my_service_bus_tcp_shared::tcp_serializers::messages_to_deliver::serialize(
            payload,
            message_content,
            attempt_no,
            &version,
        );
    }

    pub fn get_result(self) -> SendNewMessagesResult {
        if let Some(mut payload) = self.payload {
            update_messages_count(
                &mut payload,
                self.messages_count_position,
                self.messages_on_delivery.len() as i32,
            );
            return SendNewMessagesResult::Send {
                session: self.session,
                tcp_contract: TcpContract::NewMessagesServerSide(payload),
                queue_id: self.queue_id,
                messages_on_delivery: self.messages_on_delivery,
            };
        }

        SendNewMessagesResult::NothingToSend {
            queue_id: self.queue_id,
        }
    }
}

fn update_messages_count(payload: &mut Vec<u8>, messages_count_position: usize, amount: i32) {
    let size = amount.to_le_bytes();
    let dest = &mut payload[messages_count_position..messages_count_position + 4];
    dest.copy_from_slice(size.as_slice());
}
