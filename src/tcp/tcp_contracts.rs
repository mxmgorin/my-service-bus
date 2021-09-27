use my_service_bus_tcp_shared::{tcp_message_id, PacketProtVer, TcpContract};

use crate::{
    app::{AppContext, TEST_QUEUE},
    messages::{MySbMessage, MySbMessageContent},
    messages_bucket::MessagesBucket,
    topics::Topic,
};
use my_service_bus_tcp_shared::{common_serializers::*, deserializers::serialize_long};

pub async fn compile_messages_delivery_contract(
    process_id: i64,
    app: &AppContext,
    messages_to_deliver: &MessagesBucket,
    topic: &Topic,
    queue_id: &str,
    subscriber_id: i64,
) -> TcpContract {
    let mut result = Vec::new();

    let versions = app
        .sessions
        .get_packet_and_protocol_version(process_id, subscriber_id, tcp_message_id::NEW_MESSAGE)
        .await;

    if queue_id == TEST_QUEUE {
        println!("NEW_MESSAGE Packet version {:?}", versions);
    }

    result.push(tcp_message_id::NEW_MESSAGE);
    serialize_pascal_string(&mut result, topic.topic_id.as_str());
    serialize_pascal_string(&mut result, queue_id);
    serialize_long(&mut result, subscriber_id, &versions);

    serialize_messages(&mut result, &versions, messages_to_deliver).await;

    TcpContract::NewMessagesServerSide(result)
}

async fn serialize_messages(
    result: &mut Vec<u8>,
    ver: &PacketProtVer,
    messages_to_deliver: &MessagesBucket,
) {
    let messages_count = messages_to_deliver.messages_count() as i32;

    serialize_i32(result, messages_count);

    for page in &messages_to_deliver.pages {
        let read_access = page.page.data.read().await;

        for msg_id in &page.ids {
            let msg_result = read_access.messages.get(&msg_id);

            if let Some(my_sb_message) = msg_result {
                if let MySbMessage::Loaded(msg) = my_sb_message {
                    let msg_to_send = page.messages.get(&msg_id).unwrap();

                    serialize_message(result, msg, msg_to_send.attempt_no, &ver);
                } else {
                    println!("Message is not loaded. Reason {:?}", my_sb_message)
                }
            } else {
                println!("Message not found to pack {:?}", msg_id)
            }
        }
    }
}

pub fn serialize_message(
    dest: &mut Vec<u8>,
    msg: &MySbMessageContent,
    attempt_no: i32,
    ver: &PacketProtVer,
) {
    serialize_long(dest, msg.id, ver);

    if ver.packet_version == 1 {
        serialize_i32(dest, attempt_no);
    }
    serialize_byte_array(dest, msg.content.as_slice());
}
