use crate::{messages::MySbMessageContent, sessions::PacketProtVer};

use super::common_serializers::*;

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

pub fn serialize_long(payload: &mut Vec<u8>, value: i64, ver: &PacketProtVer) {
    if ver.protocol_version < 2 {
        serialize_i32(payload, value as i32);
    } else {
        serialize_i64(payload, value);
    }
}
